use mio::Token;

use crate::application::wire::{BackendKeyData, BackendMessage, WireFrame, WireMessage};
use crate::domain::pool::reactor_state::{BackendId, ClientId};

use super::socket::zero_backend_key_frame;
use super::types::{
    BackendMode, ClientMode, ReactorRuntime, StartupReplay, DISCARD_ALL_QUERY_FRAME,
};

impl ReactorRuntime {
    pub(super) fn handle_initial_backend_frame(
        &mut self,
        token: Token,
        client: ClientId,
        frame: WireFrame,
    ) {
        let raw = frame.bytes.to_vec();
        let Some(client_token) = self.client_token(client) else {
            self.close_backend(token);
            return;
        };
        self.queue_client(client_token, &raw);

        let is_ready = matches!(
            &frame.message,
            WireMessage::Backend(BackendMessage::ReadyForQuery(_))
        );
        let is_error = matches!(
            &frame.message,
            WireMessage::Backend(BackendMessage::ErrorResponse(_))
        );
        let requires_frontend_auth = matches!(
            &frame.message,
            WireMessage::Backend(
                BackendMessage::AuthenticationCleartextPassword(_)
                    | BackendMessage::AuthenticationMd5Password(_)
                    | BackendMessage::AuthenticationSasl(_)
                    | BackendMessage::AuthenticationSaslContinue(_)
            )
        );
        if let Some(backend) = self.backends.get_mut(&token) {
            if let BackendMode::InitialHandshake { replay, .. } = &mut backend.mode {
                match &frame.message {
                    WireMessage::Backend(BackendMessage::AuthenticationOk(_)) => {
                        replay.saw_auth_ok = true;
                        replay.frames.push(raw.clone());
                        replay.messages.push(BackendMessage::AuthenticationOk(
                            crate::application::wire::AuthenticationOk,
                        ));
                    }
                    WireMessage::Backend(BackendMessage::BackendKeyData(_))
                        if replay.saw_auth_ok =>
                    {
                        replay.frames.push(zero_backend_key_frame());
                        replay
                            .messages
                            .push(BackendMessage::BackendKeyData(BackendKeyData {
                                process_id: 0,
                                secret_key: 0,
                            }));
                    }
                    WireMessage::Backend(
                        BackendMessage::AuthenticationCleartextPassword(_)
                        | BackendMessage::AuthenticationMd5Password(_)
                        | BackendMessage::AuthenticationSasl(_)
                        | BackendMessage::AuthenticationSaslContinue(_),
                    ) => {
                        replay.replayable = false;
                        replay.frames.clear();
                        replay.messages.clear();
                    }
                    WireMessage::Backend(message) if replay.saw_auth_ok => {
                        replay.frames.push(raw.clone());
                        replay.messages.push(message.clone());
                    }
                    _ => {}
                }
            }
        }
        if is_error {
            self.close_backend(token);
            return;
        }
        if requires_frontend_auth {
            if let Some(client) = self.clients.get_mut(&client_token) {
                client.mode = ClientMode::Handshaking {
                    backend: BackendId(token.0 as u64),
                    awaiting_auth: true,
                };
            }
            self.resume_buffered_client_frames(client_token);
            return;
        }
        if !is_ready {
            return;
        }

        let replay = self
            .backends
            .get_mut(&token)
            .and_then(|backend| match &mut backend.mode {
                BackendMode::InitialHandshake { replay, .. } => Some(std::mem::take(replay)),
                _ => None,
            });
        let Some(replay) = replay else {
            self.close_backend(token);
            return;
        };
        let startup = self
            .clients
            .get(&client_token)
            .and_then(|entry| entry.startup.clone());
        let Some(startup) = startup else {
            self.close_backend(token);
            return;
        };
        if replay.replayable
            && replay.saw_auth_ok
            && self.startup_replays.len() < 64
            && !self
                .startup_replays
                .iter()
                .any(|entry| entry.startup == startup)
        {
            self.pool
                .publish_startup_replay(startup.clone(), replay.messages.clone());
            self.startup_replays.push(StartupReplay {
                startup,
                frames: replay.frames,
            });
        }
        if let Some(entry) = self.clients.get_mut(&client_token) {
            entry.mode = ClientMode::Idle;
            entry.wait_deadline = None;
        }
        if let Some(entry) = self.backends.get_mut(&token) {
            entry.mode = BackendMode::Resetting;
        }
        self.state.add_resetting_backend(BackendId(token.0 as u64));
        self.queue_backend(token, DISCARD_ALL_QUERY_FRAME);
        self.serve_startup_waiters();
        self.resume_buffered_client_frames(client_token);
    }

    pub(super) fn handle_bootstrap_backend_frame(
        &mut self,
        token: Token,
        client: ClientId,
        frame: WireFrame,
    ) {
        let challenge_or_error = matches!(
            &frame.message,
            WireMessage::Backend(
                BackendMessage::AuthenticationCleartextPassword(_)
                    | BackendMessage::AuthenticationMd5Password(_)
                    | BackendMessage::AuthenticationSasl(_)
                    | BackendMessage::AuthenticationSaslContinue(_)
                    | BackendMessage::ErrorResponse(_)
            )
        );
        if challenge_or_error {
            self.close_backend(token);
            return;
        }
        if let Some(backend) = self.backends.get_mut(&token) {
            if let BackendMode::Bootstrap { saw_auth_ok, .. } = &mut backend.mode {
                if matches!(
                    &frame.message,
                    WireMessage::Backend(BackendMessage::AuthenticationOk(_))
                ) {
                    *saw_auth_ok = true;
                }
            }
        }
        let ready = matches!(
            &frame.message,
            WireMessage::Backend(BackendMessage::ReadyForQuery(_))
        );
        if !ready {
            return;
        }
        let ready_for_use = matches!(
            self.backends.get(&token).map(|backend| &backend.mode),
            Some(BackendMode::Bootstrap {
                saw_auth_ok: true,
                ..
            })
        );
        if !ready_for_use {
            self.close_backend(token);
            return;
        }
        if let Some(backend) = self.backends.get_mut(&token) {
            backend.mode = BackendMode::Idle;
        }
        let action = self.state.add_clean_backend(BackendId(token.0 as u64));
        self.drive_action(action);
        self.publish_stats();
        let _ = client;
    }
}
