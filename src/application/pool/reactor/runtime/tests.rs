use mio::Token;

use super::socket::{recycle_unregistered_token, resolve_backend_address};
use super::types::FIRST_SOCKET_TOKEN;

#[test]
fn reactor_backend_address_is_cached_before_thread_start() {
    let endpoint = crate::application::proxy::config::BackendEndpointConfig {
        host: "127.0.0.1".into(),
        port: 5432,
    };
    assert_eq!(
        resolve_backend_address(&endpoint).unwrap(),
        "127.0.0.1:5432".parse().unwrap()
    );
}

#[test]
fn failed_backend_registration_recycles_token() {
    let mut free_tokens = Vec::new();
    recycle_unregistered_token(&mut free_tokens, Token(FIRST_SOCKET_TOKEN));
    assert_eq!(free_tokens.pop(), Some(FIRST_SOCKET_TOKEN));
}
