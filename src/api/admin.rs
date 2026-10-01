pub use crate::interfaces::admin::{
    build_router, drain_on_shutdown_signal, wire_server_tcp_drain, AdminState, DrainResponse,
    NamedPool, PoolListResponse, PoolStatsResponse, ReadyzResponse, ADMIN_ROUTES,
};
