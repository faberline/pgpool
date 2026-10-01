use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bytes::Bytes;
use mio::net::TcpStream as MioTcpStream;
use mio::{Poll, Token, Waker};
use server_lifecycle::ConnectionPermit;
use tokio::sync::oneshot;

use crate::application::pool::{BackendPool, PoolConfig};
use crate::application::wire::{BackendMessage, FrameReader, RelayFrame, StartupMessage};
use crate::domain::pool::reactor_state::{BackendId, ClientId, ReactorState};

pub(super) const INGRESS_TOKEN: Token = Token(0);
pub(super) const FIRST_SOCKET_TOKEN: usize = 1;
pub(super) const DISCARD_ALL_QUERY_FRAME: &[u8] = b"Q\0\0\0\x10DISCARD ALL\0";

/// A frontend accepted by the shared Tokio listener but not yet registered
/// with the transaction readiness owner.
pub(super) struct IncomingFrontend {
    pub(super) stream: std::net::TcpStream,
    pub(super) permit: ConnectionPermit,
    pub(super) completion: oneshot::Sender<()>,
}

pub(super) struct Ingress {
    pub(super) pending: Mutex<VecDeque<IncomingFrontend>>,
    pub(super) wake: Arc<Waker>,
    pub(super) stopping: AtomicBool,
}

pub(super) struct ReactorRuntime {
    pub(super) poll: Poll,
    pub(super) ingress: Arc<Ingress>,
    pub(super) pool: BackendPool,
    pub(super) config: PoolConfig,
    pub(super) backend_address: SocketAddr,
    pub(super) events: mio::Events,
    pub(super) ready_events: Vec<(Token, bool, bool)>,
    pub(super) dirty_client_interests: Vec<Token>,
    pub(super) dirty_backend_interests: Vec<Token>,
    pub(super) next_token: usize,
    pub(super) free_tokens: Vec<usize>,
    pub(super) retired_tokens: Vec<usize>,
    pub(super) next_deadline_epoch: u64,
    pub(super) clients: TokenSlots<ReactorClient>,
    pub(super) backends: TokenSlots<ReactorBackend>,
    pub(super) deadlines: VecDeque<(Instant, usize, u64)>,
    pub(super) state: ReactorState,
    pub(super) startup_replays: Vec<StartupReplay>,
    pub(super) startup_waiters: VecDeque<ClientId>,
    pub(super) stats_dirty: bool,
}

pub(super) struct TokenSlots<T> {
    pub(super) slots: Vec<Option<T>>,
    pub(super) len: usize,
}

impl<T> TokenSlots<T> {
    pub(super) fn new() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
        }
    }

    pub(super) fn get(&self, token: &Token) -> Option<&T> {
        self.slots.get(token.0).and_then(Option::as_ref)
    }

    pub(super) fn get_mut(&mut self, token: &Token) -> Option<&mut T> {
        self.slots.get_mut(token.0).and_then(Option::as_mut)
    }

    pub(super) fn contains_key(&self, token: &Token) -> bool {
        self.get(token).is_some()
    }

    pub(super) fn insert(&mut self, token: Token, value: T) -> Option<T> {
        if self.slots.len() <= token.0 {
            self.slots.resize_with(token.0 + 1, || None);
        }
        let old = self.slots[token.0].replace(value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    pub(super) fn remove(&mut self, token: &Token) -> Option<T> {
        let value = self.slots.get_mut(token.0).and_then(Option::take);
        if value.is_some() {
            self.len -= 1;
        }
        value
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(Option::as_ref)
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }
}

pub(super) struct ReactorClient {
    pub(super) stream: MioTcpStream,
    pub(super) reader: FrameReader,
    pub(super) _permit: ConnectionPermit,
    pub(super) completion: oneshot::Sender<()>,
    pub(super) mode: ClientMode,
    pub(super) startup: Option<StartupMessage>,
    pub(super) pending_first: Option<RelayFrame>,
    pub(super) output: OutputQueue,
    pub(super) wait_deadline: Option<Instant>,
    pub(super) deadline_epoch: u64,
    pub(super) interest_dirty: bool,
    pub(super) registered: bool,
}

#[derive(Debug)]
pub(super) enum ClientMode {
    Startup,
    StartupWaiting,
    Handshaking {
        backend: BackendId,
        awaiting_auth: bool,
    },
    Idle,
    Waiting,
    Active {
        backend: BackendId,
        pending_next: Option<RelayFrame>,
        /// A `ReadyForQuery(InTransaction)` is waiting for its next frontend
        /// frame. That frame belongs to this same backend immediately.
        backend_ready_for_frontend: bool,
    },
    Closing,
}

pub(super) struct ReactorBackend {
    pub(super) stream: MioTcpStream,
    pub(super) reader: FrameReader,
    pub(super) mode: BackendMode,
    pub(super) output: OutputQueue,
    pub(super) interest_dirty: bool,
    pub(super) registered: bool,
}

#[derive(Debug)]
pub(super) enum BackendMode {
    Connecting(ConnectPurpose),
    InitialHandshake {
        client: ClientId,
        replay: StartupCapture,
    },
    Bootstrap {
        client: ClientId,
        saw_auth_ok: bool,
    },
    Active {
        client: ClientId,
    },
    Resetting,
    Idle,
}

pub(super) struct OutputQueue {
    pub(super) chunks: VecDeque<Bytes>,
    pub(super) front_offset: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BackendFrameTarget {
    Initial(ClientId),
    Bootstrap(ClientId),
    Active(ClientId),
    Resetting,
    Idle,
}

#[derive(Debug, Clone)]
pub(super) enum ConnectPurpose {
    Initial { client: ClientId, startup: Bytes },
    Bootstrap { client: ClientId, startup: Bytes },
}

#[derive(Debug)]
pub(super) struct StartupCapture {
    pub(super) frames: Vec<Vec<u8>>,
    pub(super) messages: Vec<BackendMessage>,
    pub(super) saw_auth_ok: bool,
    pub(super) replayable: bool,
}

impl Default for StartupCapture {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            messages: Vec::new(),
            saw_auth_ok: false,
            replayable: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct StartupReplay {
    pub(super) startup: StartupMessage,
    pub(super) frames: Vec<Vec<u8>>,
}
