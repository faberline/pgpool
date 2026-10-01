use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointProvider {
    PlainPostgres,
    CloudSql,
    AlloyDb,
}

/// Transport selected only for control-plane connection discovery. Pgpool's
/// PostgreSQL wire data plane remains independent of this policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryTlsMode {
    NoTls,
    SystemRoots,
}

pub const fn discovery_tls_mode(provider: EndpointProvider) -> DiscoveryTlsMode {
    match provider {
        EndpointProvider::PlainPostgres => DiscoveryTlsMode::NoTls,
        EndpointProvider::CloudSql | EndpointProvider::AlloyDb => DiscoveryTlsMode::SystemRoots,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointRole {
    Primary,
    ReadPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteEndpoint {
    pub name: String,
    pub provider: EndpointProvider,
    pub role: EndpointRole,
    pub configured_ceiling: Option<u32>,
    /// Optional PEM root bundle loaded from an operator-managed Secret for a
    /// managed PostgreSQL endpoint with a private CA.
    pub tls_ca_pem: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAdvisory {
    /// Optional provider control-plane cap. It can only reduce runtime capacity.
    pub max_connections: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConnectionFacts {
    pub max_connections: u32,
    pub superuser_reserved_connections: u32,
    pub total_connections: u32,
    pub pgpool_connections: u32,
}

impl RuntimeConnectionFacts {
    pub fn non_pgpool_connections(self) -> u32 {
        self.total_connections
            .saturating_sub(self.pgpool_connections)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionFacts {
    pub endpoint: RemoteEndpoint,
    pub runtime: RuntimeConnectionFacts,
    pub advisory: ProviderAdvisory,
    pub effective_max_connections: u32,
    pub non_pgpool_connections: u32,
}

pub fn effective_connection_limit(
    runtime_max: u32,
    configured_ceiling: Option<u32>,
    advisory_max: Option<u32>,
) -> u32 {
    [Some(runtime_max), configured_ceiling, advisory_max]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(runtime_max)
}

pub(crate) fn allocatable_connection_limit(
    runtime_max: u32,
    configured_ceiling: Option<u32>,
    advisory_max: Option<u32>,
    superuser_reserved_connections: u32,
) -> u32 {
    effective_connection_limit(runtime_max, configured_ceiling, advisory_max)
        .saturating_sub(superuser_reserved_connections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_values_can_only_reduce_runtime_capacity() {
        assert_eq!(effective_connection_limit(500, None, None), 500);
        assert_eq!(effective_connection_limit(500, Some(400), None), 400);
        assert_eq!(effective_connection_limit(500, None, Some(600)), 500);
        assert_eq!(effective_connection_limit(500, Some(450), Some(420)), 420);
    }

    #[test]
    fn provider_and_endpoint_role_are_independent_budget_keys() {
        let primary = RemoteEndpoint {
            name: "alloy-primary".into(),
            provider: EndpointProvider::AlloyDb,
            role: EndpointRole::Primary,
            configured_ceiling: None,
            tls_ca_pem: None,
        };
        let read_pool = RemoteEndpoint {
            name: "alloy-read-pool".into(),
            provider: EndpointProvider::AlloyDb,
            role: EndpointRole::ReadPool,
            configured_ceiling: None,
            tls_ca_pem: None,
        };
        assert_ne!(primary, read_pool);
        assert_eq!(primary.provider, EndpointProvider::AlloyDb);
    }

    #[test]
    fn non_pgpool_usage_is_saturating() {
        let facts = RuntimeConnectionFacts {
            max_connections: 100,
            superuser_reserved_connections: 3,
            total_connections: 12,
            pgpool_connections: 5,
        };
        assert_eq!(facts.non_pgpool_connections(), 7);
        assert_eq!(
            RuntimeConnectionFacts {
                pgpool_connections: 20,
                ..facts
            }
            .non_pgpool_connections(),
            0
        );
    }

    #[test]
    fn effective_capacity_excludes_superuser_reserved_slots() {
        assert_eq!(allocatable_connection_limit(100, None, None, 3), 97);
        assert_eq!(allocatable_connection_limit(100, Some(90), Some(95), 3), 87);
        assert_eq!(allocatable_connection_limit(2, None, None, 3), 0);
    }
}
// </HANDWRITE>
