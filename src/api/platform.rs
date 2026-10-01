pub use crate::domain::platform::{
    discovery_tls_mode, effective_connection_limit, ConnectionFacts, DiscoveryTlsMode,
    EndpointProvider, EndpointRole, ProviderAdvisory, RemoteEndpoint, RuntimeConnectionFacts,
};
pub use crate::infrastructure::platform::discovery::{
    discover_connection_facts, ConnectionDiscoveryError,
};
