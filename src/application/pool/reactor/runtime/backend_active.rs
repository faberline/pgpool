use std::time::Instant;

use mio::Token;

use crate::application::wire::{
    BackendMessage, RelayFrame, RelayFrameKind, TransactionStatus, WireFrame, WireMessage,
};
use crate::domain::pool::reactor_state::{BackendId, ClientId};

use super::types::{BackendMode, ClientMode, ReactorRuntime};

impl ReactorRuntime {
    pub(super) fn handle_active_backend_frame(
        &mut self,
        token: Token,
        client: ClientId,
        frame: RelayFrame,
    ) {
        let Some(client_token) = self.client_token(client) else {
            self.close_backend(token);
            return;
        };
        let ready = match frame.kind {
            RelayFrameKind::BackendReady(status) => Some(status),
            _ => None,
        };
        self.queue_client_owned(client_token, frame.bytes);
        let Some(ready) = ready else {
            return;
        };
        let backend = BackendId(token.0 as u64);
        if ready == TransactionStatus::Idle {
            let pending = self
                .clients
                .get(&client_token)
                .and_then(|entry| match &entry.mode {
                    ClientMode::Active { pending_next, .. } => pending_next.clone(),
                    _ => None,
                });
            let has_pending = pending.is_some();
            let deadline = (pending.is_some()).then(|| Instant::now() + self.queue_wait_timeout());
            if let Some(entry) = self.clients.get_mut(&client_token) {
                if let Some(next) = pending {
                    entry.pending_first = Some(next);
                    entry.mode = ClientMode::Waiting;
                } else {
                    entry.mode = ClientMode::Idle;
                }
            }
            self.set_client_deadline(client_token, deadline);
            let action = self
                .state
                .transaction_ready_idle(client, backend, has_pending);
            self.drive_action(action);
            if has_pending {
                self.update_client_interest(client_token);
            } else {
                self.resume_buffered_client_frames(client_token);
            }
        } else if let Some(next) = self.clients.get_mut(&client_token).and_then(|entry| {
            if let ClientMode::Active { pending_next, .. } = &mut entry.mode {
                pending_next.take()
            } else {
                None
            }
        }) {
            self.queue_backend_owned(token, next.bytes);
            self.update_client_interest(client_token);
        } else if let Some(entry) = self.clients.get_mut(&client_token) {
            if let ClientMode::Active {
                backend_ready_for_frontend,
                ..
            } = &mut entry.mode
            {
                *backend_ready_for_frontend = true;
            }
            self.update_client_interest(client_token);
        }
    }

    pub(super) fn handle_reset_backend_frame(&mut self, token: Token, frame: WireFrame) {
        match frame.message {
            WireMessage::Backend(BackendMessage::ReadyForQuery(ready))
                if ready.status == TransactionStatus::Idle =>
            {
                if let Some(backend) = self.backends.get_mut(&token) {
                    backend.mode = BackendMode::Idle;
                }
                let backend = BackendId(token.0 as u64);
                let action = self.state.reset_ready_idle(backend);
                self.drive_action(action);
                self.publish_stats();
            }
            WireMessage::Backend(BackendMessage::ErrorResponse(_)) => self.close_backend(token),
            _ => {}
        }
    }
}
