use std::sync::Once;

use thiserror::Error;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::domain::platform::{
    allocatable_connection_limit, discovery_tls_mode, ConnectionFacts, DiscoveryTlsMode,
    ProviderAdvisory, RemoteEndpoint, RuntimeConnectionFacts,
};

#[derive(Debug, Error)]
pub enum ConnectionDiscoveryError {
    #[error("failed to configure discovery TLS: {0}")]
    TlsConfiguration(String),
    #[error("failed to connect to remote PostgreSQL endpoint: {0}")]
    Connect(#[source] tokio_postgres::Error),
    #[error("remote PostgreSQL discovery query failed: {0}")]
    Query(#[source] tokio_postgres::Error),
    #[error("remote max_connections value is not a positive integer: {0}")]
    InvalidMaxConnections(String),
    #[error("remote superuser_reserved_connections value is not a non-negative integer: {0}")]
    InvalidSuperuserReservedConnections(String),
    #[error("remote connection count is outside the supported u32 range: {0}")]
    InvalidConnectionCount(i64),
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in discovery.rs is hand-written pending codegen support">
/// Query the live endpoint. Provider SDK metadata is accepted only as an
/// advisory cap and never substitutes for this runtime result.
pub async fn discover_connection_facts(
    endpoint: RemoteEndpoint,
    postgres: tokio_postgres::Config,
    advisory: ProviderAdvisory,
) -> Result<ConnectionFacts, ConnectionDiscoveryError> {
    let client = match discovery_tls_mode(endpoint.provider) {
        DiscoveryTlsMode::NoTls => {
            let (client, connection) = postgres
                .connect(tokio_postgres::NoTls)
                .await
                .map_err(ConnectionDiscoveryError::Connect)?;
            tokio::spawn(async move {
                if let Err(error) = connection.await {
                    tracing::warn!(%error, "remote PostgreSQL discovery connection ended");
                }
            });
            client
        }
        DiscoveryTlsMode::SystemRoots => {
            let tls =
                MakeRustlsConnect::new(discovery_rustls_config(endpoint.tls_ca_pem.as_deref())?);
            let (client, connection) = postgres
                .connect(tls)
                .await
                .map_err(ConnectionDiscoveryError::Connect)?;
            tokio::spawn(async move {
                if let Err(error) = connection.await {
                    tracing::warn!(%error, "remote PostgreSQL TLS discovery connection ended");
                }
            });
            client
        }
    };

    let row = client
        .query_one(
            "SELECT current_setting('max_connections') AS max_connections, \
                    current_setting('superuser_reserved_connections') AS superuser_reserved_connections, \
                    count(*) FILTER (WHERE backend_type = 'client backend')::bigint AS total_connections, \
                    count(*) FILTER (WHERE backend_type = 'client backend' \
                        AND application_name LIKE 'pgpool-%')::bigint \
                        AS pgpool_connections \
             FROM pg_stat_activity",
            &[],
        )
        .await
        .map_err(ConnectionDiscoveryError::Query)?;

    let runtime_max_raw: String = row.get("max_connections");
    let runtime_max = runtime_max_raw
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| ConnectionDiscoveryError::InvalidMaxConnections(runtime_max_raw))?;
    let superuser_reserved_raw: String = row.get("superuser_reserved_connections");
    let superuser_reserved_connections = superuser_reserved_raw.parse::<u32>().map_err(|_| {
        ConnectionDiscoveryError::InvalidSuperuserReservedConnections(superuser_reserved_raw)
    })?;
    let total_connections = count_to_u32(row.get("total_connections"))?;
    let pgpool_connections = count_to_u32(row.get("pgpool_connections"))?;
    let runtime = RuntimeConnectionFacts {
        max_connections: runtime_max,
        superuser_reserved_connections,
        total_connections,
        pgpool_connections,
    };
    let effective_max_connections = allocatable_connection_limit(
        runtime.max_connections,
        endpoint.configured_ceiling,
        advisory.max_connections,
        runtime.superuser_reserved_connections,
    );

    Ok(ConnectionFacts {
        endpoint,
        runtime,
        advisory,
        effective_max_connections,
        non_pgpool_connections: runtime.non_pgpool_connections(),
    })
}
// </HANDWRITE>

fn discovery_rustls_config(
    tls_ca_pem: Option<&[u8]>,
) -> Result<rustls::ClientConfig, ConnectionDiscoveryError> {
    static INSTALL_CRYPTO_PROVIDER: Once = Once::new();
    INSTALL_CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
    let certificates = rustls_native_certs::load_native_certs();
    let mut roots = rustls::RootCertStore::empty();
    for certificate in certificates.certs {
        roots.add(certificate).map_err(|error| {
            ConnectionDiscoveryError::TlsConfiguration(format!("add system root: {error}"))
        })?;
    }
    if let Some(pem) = tls_ca_pem {
        let mut reader = std::io::Cursor::new(pem);
        let mut added = 0;
        for certificate in rustls_pemfile::certs(&mut reader) {
            let certificate = certificate.map_err(|error| {
                ConnectionDiscoveryError::TlsConfiguration(format!(
                    "read configured CA PEM: {error}"
                ))
            })?;
            roots.add(certificate).map_err(|error| {
                ConnectionDiscoveryError::TlsConfiguration(format!("add configured CA: {error}"))
            })?;
            added += 1;
        }
        if added == 0 {
            return Err(ConnectionDiscoveryError::TlsConfiguration(
                "configured TLS CA Secret contains no PEM certificates".into(),
            ));
        }
    }
    if roots.is_empty() {
        return Err(ConnectionDiscoveryError::TlsConfiguration(
            "no system trust roots available for managed PostgreSQL discovery".into(),
        ));
    }
    Ok(rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth())
}

fn count_to_u32(value: i64) -> Result<u32, ConnectionDiscoveryError> {
    value
        .try_into()
        .map_err(|_| ConnectionDiscoveryError::InvalidConnectionCount(value))
}
