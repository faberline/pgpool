use mio::Token;

use crate::domain::pool::reactor_state::{BackendId, ClientId, TransactionAction};

use super::types::{
    BackendMode, ClientMode, ConnectPurpose, ReactorRuntime, DISCARD_ALL_QUERY_FRAME,
};

impl ReactorRuntime {
    pub(super) fn drive_action(&mut self, action: TransactionAction) {
        match action {
            TransactionAction::Assign { client, backend } => {
                let (Some(client_token), Some(backend_token)) =
                    (self.client_token(client), self.backend_token(backend))
                else {
                    return;
                };
                if self
                    .clients
                    .get(&client_token)
                    .is_some_and(|entry| matches!(entry.mode, ClientMode::Closing))
                {
                    // Expiry removes the scheduler entry before it can
                    // produce an Assign action. If an action was already
                    // prepared, discard it without dropping the queued
                    // ErrorResponse; returning the backend to the scheduler
                    // lets another live waiter use it.
                    if let Some(entry) = self.clients.get_mut(&client_token) {
                        entry.pending_first = None;
                    }
                    let _ = self.state.remove_backend(backend);
                    let retry = self.state.add_clean_backend(backend);
                    self.drive_action(retry);
                    return;
                }
                let first = self
                    .clients
                    .get_mut(&client_token)
                    .and_then(|entry| entry.pending_first.take());
                let Some(first) = first else {
                    self.close_backend(backend_token);
                    return;
                };
                if let Some(client_entry) = self.clients.get_mut(&client_token) {
                    client_entry.mode = ClientMode::Active {
                        backend,
                        pending_next: None,
                        backend_ready_for_frontend: false,
                    };
                    client_entry.wait_deadline = None;
                }
                if let Some(backend_entry) = self.backends.get_mut(&backend_token) {
                    backend_entry.mode = BackendMode::Active { client };
                }
                self.queue_backend_owned(backend_token, first.bytes);
                self.update_client_interest(client_token);
                self.resume_buffered_client_frames(client_token);
                self.publish_stats();
            }
            TransactionAction::Reset { backend } => {
                if let Some(token) = self.backend_token(backend) {
                    if let Some(entry) = self.backends.get_mut(&token) {
                        entry.mode = BackendMode::Resetting;
                    }
                    self.queue_backend(token, DISCARD_ALL_QUERY_FRAME);
                    self.publish_stats();
                }
            }
            TransactionAction::Queued { .. } | TransactionAction::Idle { .. } => {}
        }
    }

    pub(super) fn close_client(&mut self, token: Token) {
        let Some(mut client) = self.clients.remove(&token) else {
            return;
        };
        let direct_backend = match client.mode {
            ClientMode::Handshaking { backend, .. } | ClientMode::Active { backend, .. } => {
                Some(backend)
            }
            _ => None,
        };
        if client.registered {
            let _ = self.poll.registry().deregister(&mut client.stream);
        }
        let id = ClientId(token.0 as u64);
        self.startup_waiters.retain(|waiting| *waiting != id);
        let _ = client.completion.send(());
        let state_backend = self.state.remove_client(id);
        if let Some(backend) = direct_backend.or(state_backend) {
            if let Some(backend_token) = self.backend_token(backend) {
                self.close_backend(backend_token);
            }
        }
        self.retired_tokens.push(token.0);
        self.publish_stats();
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1880" reason="logic section in runtime.rs is hand-written pending codegen support">
    pub(super) fn close_backend(&mut self, token: Token) {
        let Some(mut backend) = self.backends.remove(&token) else {
            return;
        };
        let direct_client = match &backend.mode {
            BackendMode::Connecting(ConnectPurpose::Initial { client, .. })
            | BackendMode::InitialHandshake { client, .. }
            | BackendMode::Active { client } => Some(*client),
            // A bootstrap is speculative capacity for a client that remains
            // Waiting in ReactorState. Auth-required backends cannot accept
            // it without a frontend password exchange, so discarding this
            // backend must not disconnect the healthy waiter.
            BackendMode::Connecting(ConnectPurpose::Bootstrap { .. })
            | BackendMode::Bootstrap { .. }
            | BackendMode::Resetting
            | BackendMode::Idle => None,
        };
        if backend.registered {
            let _ = self.poll.registry().deregister(&mut backend.stream);
        }
        let id = BackendId(token.0 as u64);
        let state_client = self.state.remove_backend(id);
        if let Some(client) = state_client.or(direct_client) {
            if let Some(client_token) = self.client_token(client) {
                self.close_client(client_token);
            }
        }
        self.retired_tokens.push(token.0);
        self.publish_stats();
    }
    // </HANDWRITE>
}
