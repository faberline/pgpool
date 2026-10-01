use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bytes::BytesMut;
use mio::net::TcpStream as MioTcpStream;
use mio::{Interest, Poll, Token, Waker};

use crate::application::pool::{BackendPool, PoolRejectionReason};
use crate::application::wire::{FrameReader, Role};
use crate::domain::pool::reactor_state::{ClientId, ReactorState};

use super::socket::resolve_backend_address;
use super::types::{
    BackendMode, ClientMode, IncomingFrontend, Ingress, OutputQueue, ReactorClient, ReactorRuntime,
    TokenSlots, FIRST_SOCKET_TOKEN, INGRESS_TOKEN,
};
use super::TransactionReactor;

impl TransactionReactor {
    // <HANDWRITE gap="missing-generator:logic" tracker="#1881" reason="Resolve the backend endpoint before spawning the readiness thread, cache its SocketAddr, and recycle tokens after failed Mio registration.">
    /// Starts the dedicated readiness owner. Backend pool configuration and
    /// admin-stat publication are captured here; no transaction socket has
    /// crossed the boundary yet.
    pub(crate) fn start(pool: BackendPool) -> io::Result<Self> {
        // DNS is resolved on the caller's Tokio worker before the single
        // readiness owner starts. The address stays cached for this reactor
        // lifetime; constructing a new handler refreshes it.
        let backend_address = resolve_backend_address(&pool.reactor_config().endpoint)?;
        let poll = Poll::new()?;
        let wake = Arc::new(Waker::new(poll.registry(), INGRESS_TOKEN)?);
        let ingress = Arc::new(Ingress {
            pending: Mutex::new(VecDeque::new()),
            wake,
            stopping: AtomicBool::new(false),
        });
        let thread_ingress = Arc::clone(&ingress);
        thread::Builder::new()
            .name("pgpool-transaction-reactor".to_string())
            .spawn(move || {
                ReactorRuntime::new(poll, thread_ingress, pool, backend_address).run()
            })?;
        Ok(Self { ingress })
    }
    // </HANDWRITE>
}

impl ReactorRuntime {
    // <HANDWRITE gap="missing-generator:logic" tracker="#1891" reason="Use the Duration queue policy directly in reactor wait deadlines.">
    /// The reactor owns FIFO deadlines. When reserve leasing is configured,
    /// its queueWaitTimeout replaces the historical local acquire timeout;
    /// reserve grants are still consumed only from the background-updated
    /// pool cache and never by a readiness callback doing Kubernetes I/O.
    pub(super) fn queue_wait_timeout(&self) -> Duration {
        self.pool
            .reserve_policy()
            .map(|policy| policy.queue_wait_timeout)
            .unwrap_or(self.config.acquire_timeout)
    }
    // </HANDWRITE>

    pub(super) fn new(
        poll: Poll,
        ingress: Arc<Ingress>,
        pool: BackendPool,
        backend_address: SocketAddr,
    ) -> Self {
        Self {
            poll,
            ingress,
            config: pool.reactor_config(),
            backend_address,
            pool,
            events: mio::Events::with_capacity(256),
            ready_events: Vec::with_capacity(256),
            dirty_client_interests: Vec::with_capacity(256),
            dirty_backend_interests: Vec::with_capacity(64),
            next_token: FIRST_SOCKET_TOKEN,
            free_tokens: Vec::with_capacity(256),
            retired_tokens: Vec::with_capacity(256),
            next_deadline_epoch: 1,
            clients: TokenSlots::new(),
            backends: TokenSlots::new(),
            deadlines: VecDeque::new(),
            state: ReactorState::new(),
            startup_replays: Vec::new(),
            startup_waiters: VecDeque::new(),
            stats_dirty: false,
        }
    }

    pub(super) fn run(mut self) {
        loop {
            let wait_timeout = self.next_wait_timeout();
            if self.poll.poll(&mut self.events, wait_timeout).is_err() {
                break;
            }
            self.ready_events.clear();
            self.ready_events.extend(
                self.events
                    .iter()
                    .map(|event| (event.token(), event.is_readable(), event.is_writable())),
            );
            for index in 0..self.ready_events.len() {
                let (token, readable, writable) = self.ready_events[index];
                self.handle_event(token, readable, writable);
            }
            self.expire_waiters();
            self.flush_interest_updates();
            self.flush_stats();
            // Tokens retired while handling this readiness snapshot cannot be
            // reused until every event in the snapshot has been consumed.
            self.free_tokens.append(&mut self.retired_tokens);
            if self.ingress.stopping.load(Ordering::Acquire) {
                break;
            }
        }
        // The full runtime begins publishing exact ownership counts as soon as
        // it registers frontend/backend sockets. Publishing zero here keeps an
        // already-stopped owner from leaving stale admin data behind.
        self.pool.publish_reactor_stats(0, 0);
    }

    pub(super) fn drain_ingress(&mut self) {
        let pending: Vec<_> = {
            let mut pending = self.ingress.pending.lock().expect("reactor ingress lock");
            pending.drain(..).collect()
        };
        for incoming in pending {
            self.register_client(incoming);
        }
    }

    pub(super) fn handle_event(&mut self, token: Token, readable: bool, writable: bool) {
        if token == INGRESS_TOKEN {
            self.drain_ingress();
            return;
        }
        if self.clients.contains_key(&token) {
            if writable {
                self.flush_client(token);
            }
            if readable && self.clients.contains_key(&token) {
                self.read_client(token);
            }
            return;
        }
        if self.backends.contains_key(&token) {
            if writable {
                self.flush_backend(token);
            }
            if readable && self.backends.contains_key(&token) {
                self.read_backend(token);
            }
        }
    }

    pub(super) fn register_client(&mut self, incoming: IncomingFrontend) {
        let token = self.next_token();
        let id = ClientId(token.0 as u64);
        let mut stream = MioTcpStream::from_std(incoming.stream);
        if stream.set_nodelay(true).is_err()
            || self
                .poll
                .registry()
                .register(&mut stream, token, Interest::READABLE)
                .is_err()
        {
            return;
        }
        self.state.add_client(id);
        self.clients.insert(
            token,
            ReactorClient {
                stream,
                reader: FrameReader::new(Role::Frontend, &self.config.wire),
                _permit: incoming.permit,
                completion: incoming.completion,
                mode: ClientMode::Startup,
                startup: None,
                pending_first: None,
                output: OutputQueue::new(),
                wait_deadline: None,
                deadline_epoch: 0,
                interest_dirty: false,
                registered: true,
            },
        );
        self.publish_stats();
    }

    pub(super) fn publish_stats(&mut self) {
        self.stats_dirty = true;
    }

    pub(super) fn flush_stats(&mut self) {
        if !self.stats_dirty {
            return;
        }
        let active = self
            .backends
            .values()
            .filter(|backend| !matches!(backend.mode, BackendMode::Idle | BackendMode::Resetting))
            .count();
        let idle = self
            .backends
            .values()
            .filter(|backend| matches!(backend.mode, BackendMode::Idle))
            .count();
        self.pool.publish_reactor_stats(active, idle);
        self.stats_dirty = false;
    }

    pub(super) fn prune_deadlines(&mut self) {
        while let Some((deadline, token, epoch)) = self.deadlines.front().copied() {
            let valid = self.clients.get(&Token(token)).is_some_and(|client| {
                client.deadline_epoch == epoch && client.wait_deadline == Some(deadline)
            });
            if valid {
                break;
            }
            self.deadlines.pop_front();
        }
    }

    pub(super) fn set_client_deadline(&mut self, token: Token, deadline: Option<Instant>) {
        let epoch = self.next_deadline_epoch;
        self.next_deadline_epoch = self.next_deadline_epoch.wrapping_add(1).max(1);
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        client.deadline_epoch = epoch;
        client.wait_deadline = deadline;
        if let Some(deadline) = deadline {
            // Every waiter uses the same acquire timeout, so insertion time
            // is deadline order. A FIFO deadline queue avoids heap work on
            // every short-lived transaction wait while retaining exact expiry.
            self.deadlines.push_back((deadline, token.0, epoch));
        }
    }

    pub(super) fn next_wait_timeout(&mut self) -> Option<std::time::Duration> {
        self.prune_deadlines();
        let now = Instant::now();
        self.deadlines
            .front()
            .map(|(deadline, _, _)| deadline.saturating_duration_since(now))
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1879" reason="logic section in runtime.rs is hand-written pending codegen support">
    pub(super) fn expire_waiters(&mut self) {
        let now = Instant::now();
        loop {
            self.prune_deadlines();
            let Some((deadline, token_index, _)) = self.deadlines.front().copied() else {
                break;
            };
            if deadline > now {
                break;
            }
            self.deadlines.pop_front();
            let token = Token(token_index);
            let epoch = self.next_deadline_epoch;
            self.next_deadline_epoch = self.next_deadline_epoch.wrapping_add(1).max(1);
            let expired = if let Some(client) = self.clients.get_mut(&token) {
                client.wait_deadline = None;
                client.deadline_epoch = epoch;
                client.mode = ClientMode::Closing;
                client.pending_first = None;
                true
            } else {
                false
            };
            if !expired {
                continue;
            }
            // The socket remains long enough to flush the rejection, but the
            // scheduler must forget it now. Otherwise a delayed clean-backend
            // action can resurrect this Closing client and relay its stale
            // first query after the 53300 error.
            let _ = self.state.remove_client(ClientId(token.0 as u64));
            let mut error = BytesMut::new();
            PoolRejectionReason::BackendPoolSaturated
                .synthesized_error_response()
                .encode(&mut error);
            self.queue_client_owned(token, error.freeze());
        }
    }
    // </HANDWRITE>
}
