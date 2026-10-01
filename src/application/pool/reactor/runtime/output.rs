use std::collections::VecDeque;
use std::io::{self, IoSlice, Write};

use bytes::Bytes;
use mio::net::TcpStream as MioTcpStream;
use mio::{Interest, Token};

use super::client::client_can_read;
use super::types::{BackendMode, ClientMode, OutputQueue, ReactorRuntime};

impl OutputQueue {
    pub(super) fn new() -> Self {
        Self {
            chunks: VecDeque::new(),
            front_offset: 0,
        }
    }

    pub(super) fn push(&mut self, bytes: Bytes) {
        if !bytes.is_empty() {
            self.chunks.push_back(bytes);
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub(super) fn advance(&mut self, mut written: usize) {
        while written > 0 {
            let remaining = self
                .chunks
                .front()
                .map(|chunk| chunk.len() - self.front_offset)
                .expect("write cannot exceed queued bytes");
            if written < remaining {
                self.front_offset += written;
                return;
            }
            written -= remaining;
            self.chunks.pop_front();
            self.front_offset = 0;
        }
    }
}

impl ReactorRuntime {
    pub(super) fn queue_client(&mut self, token: Token, bytes: impl AsRef<[u8]>) {
        self.queue_client_owned(token, Bytes::copy_from_slice(bytes.as_ref()));
    }

    pub(super) fn queue_client_owned(&mut self, token: Token, bytes: Bytes) {
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        client.output.push(bytes);
        self.update_client_interest(token);
    }

    pub(super) fn queue_backend(&mut self, token: Token, bytes: impl AsRef<[u8]>) {
        self.queue_backend_owned(token, Bytes::copy_from_slice(bytes.as_ref()));
    }

    pub(super) fn queue_backend_owned(&mut self, token: Token, bytes: Bytes) {
        let Some(backend) = self.backends.get_mut(&token) else {
            return;
        };
        backend.output.push(bytes);
        self.update_backend_interest(token);
    }

    pub(super) fn flush_client(&mut self, token: Token) {
        let close = match self.clients.get_mut(&token) {
            Some(client) => flush_socket(&mut client.stream, &mut client.output),
            None => return,
        };
        if close {
            self.close_client(token);
            return;
        }
        let close_after_flush = matches!(
            self.clients.get(&token).map(|client| &client.mode),
            Some(ClientMode::Closing)
        ) && self
            .clients
            .get(&token)
            .is_some_and(|client| client.output.is_empty());
        if close_after_flush {
            self.close_client(token);
        } else {
            self.update_client_interest(token);
        }
    }

    pub(super) fn flush_backend(&mut self, token: Token) {
        self.complete_connect(token);
        let close = match self.backends.get_mut(&token) {
            Some(backend) => flush_socket(&mut backend.stream, &mut backend.output),
            None => return,
        };
        if close {
            self.close_backend(token);
        } else {
            self.update_backend_interest(token);
        }
    }

    pub(super) fn update_client_interest(&mut self, token: Token) {
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        if client.interest_dirty {
            return;
        }
        client.interest_dirty = true;
        self.dirty_client_interests.push(token);
    }

    pub(super) fn update_backend_interest(&mut self, token: Token) {
        let Some(backend) = self.backends.get_mut(&token) else {
            return;
        };
        if backend.interest_dirty {
            return;
        }
        backend.interest_dirty = true;
        self.dirty_backend_interests.push(token);
    }

    pub(super) fn flush_interest_updates(&mut self) {
        while let Some(token) = self.dirty_client_interests.pop() {
            let Some(client) = self.clients.get_mut(&token) else {
                continue;
            };
            client.interest_dirty = false;
            self.apply_client_interest(token);
        }
        while let Some(token) = self.dirty_backend_interests.pop() {
            let Some(backend) = self.backends.get_mut(&token) else {
                continue;
            };
            backend.interest_dirty = false;
            self.apply_backend_interest(token);
        }
    }

    pub(super) fn apply_client_interest(&mut self, token: Token) {
        let close = match self.clients.get_mut(&token) {
            Some(client) if !client.output.is_empty() => {
                flush_socket(&mut client.stream, &mut client.output)
            }
            Some(_) => false,
            None => return,
        };
        if close {
            self.close_client(token);
            return;
        }
        let close_after_flush = self.clients.get(&token).is_some_and(|client| {
            matches!(client.mode, ClientMode::Closing) && client.output.is_empty()
        });
        if close_after_flush {
            self.close_client(token);
            return;
        }
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        let want_read = client_can_read(&client.mode);
        let want_write = !client.output.is_empty();
        let interest = match (want_read, want_write) {
            (true, true) => Some(Interest::READABLE | Interest::WRITABLE),
            (true, false) => Some(Interest::READABLE),
            (false, true) => Some(Interest::WRITABLE),
            (false, false) => None,
        };
        match (client.registered, interest) {
            (true, Some(interest)) => {
                if self
                    .poll
                    .registry()
                    .reregister(&mut client.stream, token, interest)
                    .is_err()
                {
                    self.close_client(token);
                }
            }
            (false, Some(interest)) => {
                if self
                    .poll
                    .registry()
                    .register(&mut client.stream, token, interest)
                    .is_err()
                {
                    self.close_client(token);
                } else {
                    client.registered = true;
                }
            }
            (true, None) => {
                let _ = self.poll.registry().deregister(&mut client.stream);
                client.registered = false;
            }
            (false, None) => {}
        }
    }

    pub(super) fn apply_backend_interest(&mut self, token: Token) {
        let close = match self.backends.get_mut(&token) {
            Some(backend)
                if !matches!(backend.mode, BackendMode::Connecting(_))
                    && !backend.output.is_empty() =>
            {
                flush_socket(&mut backend.stream, &mut backend.output)
            }
            Some(_) => false,
            None => return,
        };
        if close {
            self.close_backend(token);
            return;
        }
        let Some(backend) = self.backends.get_mut(&token) else {
            return;
        };
        let interest = if backend.output.is_empty() {
            Interest::READABLE
        } else {
            Interest::READABLE | Interest::WRITABLE
        };
        let result = if backend.registered {
            self.poll
                .registry()
                .reregister(&mut backend.stream, token, interest)
        } else {
            self.poll
                .registry()
                .register(&mut backend.stream, token, interest)
        };
        if result.is_err() {
            self.close_backend(token);
        } else {
            backend.registered = true;
        }
    }
}

pub(super) fn flush_socket(stream: &mut MioTcpStream, output: &mut OutputQueue) -> bool {
    const MAX_IO_SLICES: usize = 16;
    while !output.is_empty() {
        let result = {
            let mut slices: [IoSlice<'_>; MAX_IO_SLICES] =
                std::array::from_fn(|_| IoSlice::new(&[]));
            let mut count = 0;
            for (index, chunk) in output.chunks.iter().take(MAX_IO_SLICES).enumerate() {
                let start = if index == 0 { output.front_offset } else { 0 };
                slices[index] = IoSlice::new(&chunk[start..]);
                count += 1;
            }
            stream.write_vectored(&slices[..count])
        };
        match result {
            Ok(0) => return true,
            Err(ref error) if error.kind() == io::ErrorKind::WriteZero => return true,
            Ok(written) => output.advance(written),
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(_) => return true,
        }
    }
    false
}
