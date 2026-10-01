use mio::Token;

use crate::application::wire::{
    BackendMessage, RelayFrame, RelayFrameKind, WireFrame, WireMessage,
};

use super::socket::drain_socket;
use super::types::{
    BackendFrameTarget, BackendMode, ConnectPurpose, ReactorRuntime, StartupCapture,
};

impl ReactorRuntime {
    pub(super) fn read_backend(&mut self, _token: Token) {
        let token = _token;
        self.complete_connect(token);
        let close = match self.backends.get_mut(&token) {
            Some(backend) => drain_socket(&mut backend.stream, &mut backend.reader),
            None => return,
        };
        if close {
            self.close_backend(token);
            return;
        }
        loop {
            let active_client = self
                .backends
                .get(&token)
                .and_then(|backend| match backend.mode {
                    BackendMode::Active { client } => Some(client),
                    _ => None,
                });
            if let Some(client) = active_client {
                let frame = match self.backends.get_mut(&token) {
                    Some(backend) => backend.reader.next_relay_frame_with_raw(),
                    None => return,
                };
                match frame {
                    Ok(Some(frame)) => self.handle_active_backend_frame(token, client, frame),
                    Ok(None) => break,
                    Err(_) => {
                        self.close_backend(token);
                        return;
                    }
                }
                if !self.backends.contains_key(&token) {
                    return;
                }
                continue;
            }
            let frame = match self.backends.get_mut(&token) {
                Some(backend) => backend.reader.next_frame_with_raw(),
                None => return,
            };
            match frame {
                Ok(Some(frame)) => self.handle_backend_frame(token, frame),
                Ok(None) => break,
                Err(_) => {
                    self.close_backend(token);
                    return;
                }
            }
            if !self.backends.contains_key(&token) {
                return;
            }
        }
        self.update_backend_interest(token);
    }

    pub(super) fn complete_connect(&mut self, token: Token) {
        let purpose = match self.backends.get(&token).map(|backend| &backend.mode) {
            Some(BackendMode::Connecting(purpose)) => purpose.clone(),
            _ => return,
        };
        let error = self
            .backends
            .get_mut(&token)
            .and_then(|backend| backend.stream.take_error().ok())
            .flatten();
        if error.is_some() {
            self.close_backend(token);
            return;
        }
        let (mode, startup) = match purpose {
            ConnectPurpose::Initial { client, startup } => (
                BackendMode::InitialHandshake {
                    client,
                    replay: StartupCapture::default(),
                },
                startup,
            ),
            ConnectPurpose::Bootstrap { client, startup } => (
                BackendMode::Bootstrap {
                    client,
                    saw_auth_ok: false,
                },
                startup,
            ),
        };
        if let Some(backend) = self.backends.get_mut(&token) {
            if backend.stream.set_nodelay(true).is_err() {
                self.close_backend(token);
                return;
            }
            backend.mode = mode;
        }
        self.queue_backend_owned(token, startup);
        self.publish_stats();
    }

    pub(super) fn handle_backend_frame(&mut self, token: Token, frame: WireFrame) {
        let mode = match self.backends.get(&token).map(|backend| &backend.mode) {
            Some(BackendMode::InitialHandshake { client, .. }) => {
                BackendFrameTarget::Initial(*client)
            }
            Some(BackendMode::Bootstrap { client, .. }) => BackendFrameTarget::Bootstrap(*client),
            Some(BackendMode::Active { client }) => BackendFrameTarget::Active(*client),
            Some(BackendMode::Resetting) => BackendFrameTarget::Resetting,
            Some(BackendMode::Idle) => BackendFrameTarget::Idle,
            Some(BackendMode::Connecting(_)) | None => return,
        };
        match mode {
            BackendFrameTarget::Initial(client) => {
                self.handle_initial_backend_frame(token, client, frame)
            }
            BackendFrameTarget::Bootstrap(client) => {
                self.handle_bootstrap_backend_frame(token, client, frame)
            }
            BackendFrameTarget::Active(client) => {
                let kind = match &frame.message {
                    WireMessage::Backend(BackendMessage::ReadyForQuery(ready)) => {
                        RelayFrameKind::BackendReady(ready.status)
                    }
                    _ => RelayFrameKind::Other,
                };
                self.handle_active_backend_frame(
                    token,
                    client,
                    RelayFrame {
                        kind,
                        bytes: frame.bytes,
                    },
                )
            }
            BackendFrameTarget::Resetting => self.handle_reset_backend_frame(token, frame),
            BackendFrameTarget::Idle => self.close_backend(token),
        }
    }
}
