use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use bytes::BytesMut;
use mio::net::TcpStream as MioTcpStream;
use mio::Token;

use crate::application::wire::{BackendKeyData, BackendMessage, FrameReader};
use crate::domain::pool::reactor_state::{BackendId, ClientId};

use super::types::ReactorRuntime;

impl ReactorRuntime {
    pub(super) fn next_token(&mut self) -> Token {
        if let Some(token) = self.free_tokens.pop() {
            return Token(token);
        }
        let token = Token(self.next_token);
        self.next_token += 1;
        token
    }

    pub(super) fn client_token(&self, client: ClientId) -> Option<Token> {
        let token = Token(client.0 as usize);
        self.clients.contains_key(&token).then_some(token)
    }

    pub(super) fn backend_token(&self, backend: BackendId) -> Option<Token> {
        let token = Token(backend.0 as usize);
        self.backends.contains_key(&token).then_some(token)
    }
}

/// Writes until the kernel would block. `true` means EOF/error and asks the
/// owner to close the complete client/backend state, not merely this socket.
pub(super) fn resolve_backend_address(
    endpoint: &crate::application::proxy::config::BackendEndpointConfig,
) -> io::Result<SocketAddr> {
    (endpoint.host.as_str(), endpoint.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "backend hostname resolved no addresses",
            )
        })
}

pub(super) fn recycle_unregistered_token(free_tokens: &mut Vec<usize>, token: Token) {
    free_tokens.push(token.0);
}

/// Drains a nonblocking socket directly into its persistent parser buffer.
/// `true` classifies EOF or an I/O failure as terminal; `WouldBlock` ends the
/// current readiness turn without allocating or copying through a temporary
/// aggregate buffer.
pub(super) fn drain_socket(stream: &mut MioTcpStream, reader: &mut FrameReader) -> bool {
    loop {
        match reader.read_from_sync(stream) {
            Ok(0) => return true,
            Ok(_) => {}
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => return false,
            Err(_) => return true,
        }
    }
}

pub(super) fn zero_backend_key_frame() -> Vec<u8> {
    let mut bytes = BytesMut::new();
    BackendMessage::BackendKeyData(BackendKeyData {
        process_id: 0,
        secret_key: 0,
    })
    .encode(&mut bytes);
    bytes.to_vec()
}
