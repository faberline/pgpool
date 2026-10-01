//! Socket-readiness owner for transaction pooling.

mod backend;
mod backend_active;
mod backend_startup;
mod client;
mod close;
mod r#loop;
mod output;
mod socket;
mod startup;
mod types;

#[cfg(test)]
mod tests;

use std::io;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use server_lifecycle::ConnectionPermit;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::oneshot;

use types::{IncomingFrontend, Ingress};

/// Cloneable, one-shot handoff handle owned by [`crate::interfaces::pool::TransactionHandler`].
///
/// It intentionally exposes no acquire/release API: those operations belong
/// to the reactor's single owner, not the Tokio task that accepted a client.
#[derive(Clone)]
pub(crate) struct TransactionReactor {
    ingress: Arc<Ingress>,
}

impl std::fmt::Debug for TransactionReactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionReactor").finish_non_exhaustive()
    }
}

impl TransactionReactor {
    /// Transfers the accepted socket and its frontend budget permit to the
    /// readiness owner. `TcpStream::into_std` preserves nonblocking mode; the
    /// reactor resets it explicitly before `mio` registration so this remains
    /// correct if Tokio's conversion semantics ever change.
    pub(crate) fn handoff(
        &self,
        stream: TokioTcpStream,
        permit: ConnectionPermit,
    ) -> io::Result<oneshot::Receiver<()>> {
        let stream = stream.into_std()?;
        stream.set_nonblocking(true)?;
        let (completion, done) = oneshot::channel();
        {
            let mut pending = self.ingress.pending.lock().expect("reactor ingress lock");
            pending.push_back(IncomingFrontend {
                stream,
                permit,
                completion,
            });
        }
        self.ingress.wake.wake()?;
        Ok(done)
    }
}

impl Drop for TransactionReactor {
    fn drop(&mut self) {
        // The runtime keeps one ingress Arc. `server_tcp::serve_arc` now
        // retains its handler through drain, so seeing only this final
        // handler plus the runtime means every frontend completion task has
        // finished and the owner can stop without truncating an in-flight
        // transaction.
        if Arc::strong_count(&self.ingress) == 2 {
            self.ingress.stopping.store(true, Ordering::Release);
            let _ = self.ingress.wake.wake();
        }
    }
}
