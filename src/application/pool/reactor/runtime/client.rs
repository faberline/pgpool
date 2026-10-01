use std::time::Instant;

use bytes::BytesMut;
use mio::Token;

use crate::application::pool::rejection::extended_query_rejection;
use crate::application::wire::{RelayFrame, RelayFrameKind};
use crate::domain::pool::reactor_state::ClientId;

use super::socket::drain_socket;
use super::types::{ClientMode, ConnectPurpose, ReactorRuntime};

impl ReactorRuntime {
    /// Queues the transaction-mode protocol stopgap response before closing.
    /// `Closing` disables reads while the writable interest flushes the error
    /// to the frontend, so a rejected client observes ErrorResponse then EOF.
    pub(super) fn reject_extended_query(&mut self, token: Token) {
        if let Some(client) = self.clients.get_mut(&token) {
            client.mode = ClientMode::Closing;
            client.wait_deadline = None;
        }
        let mut error = BytesMut::new();
        extended_query_rejection().encode(&mut error);
        self.queue_client_owned(token, error.freeze());
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1878" reason="logic section in runtime.rs is hand-written pending codegen support">
    pub(super) fn read_client(&mut self, token: Token) {
        let close = match self.clients.get_mut(&token) {
            Some(client) => drain_socket(&mut client.stream, &mut client.reader),
            None => return,
        };
        if close {
            self.close_client(token);
            return;
        }
        self.drain_buffered_client_frames(token);
    }

    /// Re-drives complete frames that were already read from the socket before
    /// a transition made this client readable again. The caller must have
    /// changed the mode first; this helper never reads the socket itself, so it
    /// cannot bypass the wait/pending backpressure boundary.
    pub(super) fn resume_buffered_client_frames(&mut self, token: Token) {
        if self
            .clients
            .get(&token)
            .is_some_and(|client| client_can_read(&client.mode))
        {
            self.drain_buffered_client_frames(token);
        }
    }

    pub(super) fn drain_buffered_client_frames(&mut self, token: Token) {
        let id = ClientId(token.0 as u64);

        loop {
            let startup_phase = self
                .clients
                .get(&token)
                .is_some_and(|client| matches!(client.mode, ClientMode::Startup));
            if startup_phase {
                let frame = match self.clients.get_mut(&token) {
                    Some(client) => client.reader.next_frame_with_raw(),
                    None => return,
                };
                match frame {
                    Ok(Some(frame)) => self.handle_startup_frame(id, token, frame),
                    Ok(None) => break,
                    Err(_) => {
                        self.close_client(token);
                        return;
                    }
                }
            } else {
                let can_read = self
                    .clients
                    .get(&token)
                    .is_some_and(|client| client_can_read(&client.mode));
                if !can_read {
                    break;
                }
                let frame = match self.clients.get_mut(&token) {
                    Some(client) => client.reader.next_relay_frame_with_raw(),
                    None => return,
                };
                match frame {
                    Ok(Some(frame)) => {
                        self.handle_client_relay_frame(id, token, frame);
                        // A waiting/pipelined client is intentionally
                        // deregistered for reads to apply socket backpressure.
                        if !self
                            .clients
                            .get(&token)
                            .is_some_and(|client| client_can_read(&client.mode))
                        {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        self.close_client(token);
                        return;
                    }
                }
            }
            if !self.clients.contains_key(&token) {
                return;
            }
        }
        self.update_client_interest(token);
    }
    // </HANDWRITE>

    pub(super) fn handle_client_relay_frame(
        &mut self,
        id: ClientId,
        token: Token,
        frame: RelayFrame,
    ) {
        if matches!(frame.kind, RelayFrameKind::FrontendTerminate) {
            self.close_client(token);
            return;
        }
        if frame.is_extended_query() {
            self.reject_extended_query(token);
            return;
        }
        let mode = self.clients.get(&token).map(|client| match &client.mode {
            ClientMode::Idle => (0_u8, None),
            ClientMode::Handshaking {
                backend,
                awaiting_auth: true,
            } => (1, Some(*backend)),
            ClientMode::Active {
                backend,
                pending_next: None,
                backend_ready_for_frontend: true,
                ..
            } => (2, Some(*backend)),
            ClientMode::Active {
                pending_next: None, ..
            } => (3, None),
            _ => (4, None),
        });
        match mode {
            Some((0, _)) => {
                let deadline = Instant::now() + self.queue_wait_timeout();
                if let Some(client) = self.clients.get_mut(&token) {
                    client.pending_first = Some(frame);
                    client.mode = ClientMode::Waiting;
                }
                self.set_client_deadline(token, Some(deadline));
                let action = self.state.request_backend(id);
                self.drive_action(action);
                if self.backends.len() < self.config.max_backend_connections
                    && self.state.should_open_normal_backend()
                    && matches!(
                        self.clients.get(&token).map(|client| &client.mode),
                        Some(ClientMode::Waiting)
                    )
                {
                    let _ = self.open_backend(ConnectPurpose::Bootstrap {
                        client: id,
                        startup: self.startup_bytes(id),
                    });
                }
            }
            Some((1, Some(backend))) => {
                if let Some(client) = self.clients.get_mut(&token) {
                    if let ClientMode::Handshaking { awaiting_auth, .. } = &mut client.mode {
                        *awaiting_auth = false;
                    }
                }
                if let Some(backend_token) = self.backend_token(backend) {
                    self.queue_backend_owned(backend_token, frame.bytes);
                } else {
                    self.close_client(token);
                }
            }
            Some((2, Some(backend))) => {
                if let Some(client) = self.clients.get_mut(&token) {
                    if let ClientMode::Active {
                        backend_ready_for_frontend,
                        ..
                    } = &mut client.mode
                    {
                        *backend_ready_for_frontend = false;
                    }
                }
                if let Some(backend_token) = self.backend_token(backend) {
                    self.queue_backend_owned(backend_token, frame.bytes);
                } else {
                    self.close_client(token);
                }
            }
            Some((3, _)) => {
                if let Some(client) = self.clients.get_mut(&token) {
                    if let ClientMode::Active { pending_next, .. } = &mut client.mode {
                        *pending_next = Some(frame);
                    }
                }
            }
            _ => self.close_client(token),
        }
    }
}

pub(super) fn client_can_read(mode: &ClientMode) -> bool {
    matches!(
        mode,
        ClientMode::Startup
            | ClientMode::Idle
            | ClientMode::Handshaking {
                awaiting_auth: true,
                ..
            }
            | ClientMode::Active {
                pending_next: None,
                ..
            }
    )
}
