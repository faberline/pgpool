use std::time::Instant;

use bytes::{Bytes, BytesMut};
use mio::net::TcpStream as MioTcpStream;
use mio::{Interest, Token};

use crate::application::wire::{FrameReader, FrontendMessage, Role, WireFrame, WireMessage};
use crate::domain::pool::reactor_state::{BackendId, ClientId};

use super::socket::recycle_unregistered_token;
use super::types::{
    BackendMode, ClientMode, ConnectPurpose, OutputQueue, ReactorBackend, ReactorRuntime,
};

impl ReactorRuntime {
    pub(super) fn serve_startup_waiters(&mut self) {
        let waiting: Vec<_> = self.startup_waiters.drain(..).collect();
        for client in waiting {
            let Some(token) = self.client_token(client) else {
                continue;
            };
            if matches!(
                self.clients.get(&token).map(|entry| &entry.mode),
                Some(ClientMode::StartupWaiting)
            ) {
                self.admit_startup(client);
                self.resume_buffered_client_frames(token);
            }
        }
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in runtime.rs is hand-written pending codegen support">
    pub(super) fn handle_startup_frame(&mut self, id: ClientId, token: Token, frame: WireFrame) {
        match frame.message {
            WireMessage::Frontend(FrontendMessage::Ssl(_)) => self.queue_client(token, b"N"),
            WireMessage::Frontend(FrontendMessage::Startup(startup)) => {
                let startup = self.pool.normalize_backend_startup(startup);
                if let Some(client) = self.clients.get_mut(&token) {
                    client.startup = Some(startup);
                }
                self.admit_startup(id);
            }
            _ => self.close_client(token),
        }
    }
    // </HANDWRITE>

    pub(super) fn startup_bytes(&self, client: ClientId) -> Bytes {
        let mut bytes = BytesMut::new();
        if let Some(token) = self.client_token(client) {
            if let Some(startup) = self
                .clients
                .get(&token)
                .and_then(|client| client.startup.clone())
            {
                FrontendMessage::Startup(startup).encode(&mut bytes);
            }
        }
        bytes.freeze()
    }

    pub(super) fn admit_startup(&mut self, client: ClientId) {
        let Some(token) = self.client_token(client) else {
            return;
        };
        let startup = self
            .clients
            .get(&token)
            .and_then(|entry| entry.startup.clone());
        let Some(startup) = startup else {
            self.close_client(token);
            return;
        };
        if let Some(replay) = self
            .startup_replays
            .iter()
            .find(|entry| entry.startup == startup)
            .cloned()
        {
            for frame in replay.frames {
                self.queue_client_owned(token, Bytes::from(frame));
            }
            if let Some(entry) = self.clients.get_mut(&token) {
                entry.mode = ClientMode::Idle;
                entry.wait_deadline = None;
            }
            self.update_client_interest(token);
            return;
        }
        if self.backends.len() < self.config.max_backend_connections {
            if let Some(backend) = self.open_backend(ConnectPurpose::Initial {
                client,
                startup: self.startup_bytes(client),
            }) {
                if let Some(entry) = self.clients.get_mut(&token) {
                    entry.mode = ClientMode::Handshaking {
                        backend,
                        awaiting_auth: false,
                    };
                    entry.wait_deadline = None;
                }
                return;
            }
        }
        let deadline = Instant::now() + self.queue_wait_timeout();
        if let Some(entry) = self.clients.get_mut(&token) {
            entry.mode = ClientMode::StartupWaiting;
        }
        self.set_client_deadline(token, Some(deadline));
        self.startup_waiters.push_back(client);
        self.update_client_interest(token);
    }

    pub(super) fn open_backend(&mut self, purpose: ConnectPurpose) -> Option<BackendId> {
        let mut stream = match MioTcpStream::connect(self.backend_address) {
            Ok(stream) => stream,
            Err(_) => return None,
        };
        let token = self.next_token();
        if self
            .poll
            .registry()
            .register(&mut stream, token, Interest::READABLE | Interest::WRITABLE)
            .is_err()
        {
            recycle_unregistered_token(&mut self.free_tokens, token);
            return None;
        }
        let id = BackendId(token.0 as u64);
        self.backends.insert(
            token,
            ReactorBackend {
                stream,
                reader: FrameReader::new(Role::Backend, &self.config.wire),
                mode: BackendMode::Connecting(purpose),
                output: OutputQueue::new(),
                interest_dirty: false,
                registered: true,
            },
        );
        self.publish_stats();
        Some(id)
    }
}
