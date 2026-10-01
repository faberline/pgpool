use anyhow::Result;

use super::cli::ServeArgs;

// <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in pgpool.rs is hand-written pending codegen support">
/// `serve_entry` in the TD Logic flowchart: build a `TcpServerConfig` from
/// `RuntimePlan` with NO server-tcp-level `ConnectionBudget` wired in (the
/// `SessionHandler`/`TransactionHandler` enforce their own admission so a
/// rejection can write a wire-level `ErrorResponse` before closing), share
/// ONE `server_lifecycle::DrainController` between the TCP frontend and the
/// admin plane (`share_drain`), spawn the SIGTERM/SIGINT signal task
/// (`spawn_signal_task`), build the admin router (`build_admin_router`),
/// then run both planes concurrently (`run_both_planes`) until they both
/// drain and exit (R1-R7, AC1-AC4).
pub(super) async fn serve(args: ServeArgs) -> Result<()> {
    let mut plan = crate::application::runtime_plan::default_runtime_plan();
    plan.max_backend_connections = args.max_backend_connections;

    let frontend_bind = match &args.bind {
        Some(addr) => {
            let addr: std::net::SocketAddr = addr.parse()?;
            server_lifecycle::BindConfig {
                host: addr.ip(),
                port: addr.port(),
            }
        }
        None => plan.frontend_bind.clone(),
    };

    if let Some(addr) = &args.admin_bind {
        let addr: std::net::SocketAddr = addr.parse()?;
        plan.admin_bind = server_lifecycle::BindConfig {
            host: addr.ip(),
            port: addr.port(),
        };
    }
    plan.admin_h2c.drain_timeout = std::time::Duration::from_millis(args.admin_drain_timeout_ms);

    let backend_connect_timeout = std::time::Duration::from_millis(args.backend_connect_timeout_ms);
    let drain_timeout = std::time::Duration::from_millis(args.drain_timeout_ms);
    let wire = crate::application::wire::WireCodecConfig::default();

    // Shared backend pool (WI #1289): both pool modes dial/return connections
    // through the same capacity-bounded `BackendPool` (R1), and the admin
    // plane's `NamedPool` holds a clone of this SAME pool for live stats
    // (R3) — cloned before the handler match arm below consumes it.
    let pool_config = crate::application::pool::PoolConfig {
        endpoint: crate::application::proxy::config::BackendEndpointConfig {
            host: args.backend_host.clone(),
            port: args.backend_port,
        },
        max_backend_connections: plan.max_backend_connections,
        acquire_timeout: std::time::Duration::from_millis(args.pool_acquire_timeout_ms),
        backend_connect_timeout,
        wire,
    };
    let backend_application_name = format!("pgpool-{}", args.pod_name);
    let backend_pool = (if args.reserve_endpoint.is_empty() || args.reserve_pod.is_empty() {
        crate::application::pool::BackendPool::new(pool_config)
    } else {
        crate::application::pool::BackendPool::new_with_reserve(
            pool_config,
            crate::application::pool::ReserveLeaseRuntimeConfig {
                endpoint: args.reserve_endpoint.clone(),
                pod: args.reserve_pod.clone(),
                policy: crate::application::pool::ReserveLeasePolicy::from_millis(
                    args.reserve_pool_timeout_ms,
                    args.queue_wait_timeout_ms,
                    args.reserve_idle_timeout_ms,
                    args.reserve_lease_ttl_seconds,
                    args.reserve_request_chunk_size,
                ),
            },
        )
    })
    .with_backend_application_name(backend_application_name);
    let admin_backend_pool = backend_pool.clone();

    // Called exactly once: the SAME `ConnectionBudget` is shared into
    // whichever `PoolHandler` arm is built below AND into the admin
    // plane's `NamedPool` (R3 Schema: "never constructing a second
    // budget").
    let frontend_budget = plan.frontend_budget();

    let proxy_config = crate::application::proxy::config::SessionProxyConfig {
        backend: crate::application::proxy::config::BackendEndpointConfig {
            host: args.backend_host.clone(),
            port: args.backend_port,
        },
        frontend_budget: frontend_budget.clone(),
        backend_connect_timeout,
        drain_timeout,
        wire,
        backend_pool: backend_pool.clone(),
    };

    // `share_drain` (TD Logic section): one `DrainController` for the whole
    // process, cloned into `TcpServerConfig.drain`, `AdminState`, and the
    // signal task below — never a second, independent controller (R2, R7).
    let drain = server_lifecycle::DrainController::new();

    let server_config = server_tcp::TcpServerConfig::new(frontend_bind)
        .with_socket_options(plan.frontend_socket)
        .with_drain_timeout(drain_timeout);
    let server_config = crate::interfaces::admin::wire_server_tcp_drain(server_config, &drain);

    // `PoolHandler` dispatch (TD Schema section): selected once at process
    // start from `RuntimePlan::pool_mode`, never re-evaluated per
    // connection.
    let handler = match plan.pool_mode {
        crate::application::runtime_plan::PoolMode::Session => {
            crate::interfaces::pool::PoolHandler::Session(
                crate::interfaces::proxy::SessionHandler::new(proxy_config),
            )
        }
        crate::application::runtime_plan::PoolMode::Transaction => {
            crate::interfaces::pool::PoolHandler::Transaction(
                crate::interfaces::pool::TransactionHandler::new(
                    crate::interfaces::pool::TransactionProxyConfig {
                        frontend_budget: frontend_budget.clone(),
                        backend_pool,
                        wire,
                        drain_timeout,
                    },
                ),
            )
        }
    };

    // `build_admin_router` (TD Logic section): one named pool per process
    // today (R3 Schema note), labeled via `--pool-name`/`PGPOOL_POOL_NAME`.
    let admin_state = crate::interfaces::admin::AdminState::new(
        drain.clone(),
        vec![crate::interfaces::admin::NamedPool {
            name: args.pool_name.clone(),
            mode: plan.pool_mode.clone(),
            budget: frontend_budget,
            pool: admin_backend_pool,
        }],
    );
    let admin_router = crate::interfaces::admin::build_router(admin_state);

    // Subscribe both serving planes before the signal task can publish a
    // drain. `DrainSignal::changed` also observes an already-published state,
    // so a drain during either bind below resolves both shutdown futures.
    let tcp_shutdown = {
        let mut signal = drain.signal();
        async move {
            signal.changed().await;
        }
    };
    let admin_shutdown = {
        let mut signal = drain.signal();
        async move {
            signal.changed().await;
        }
    };

    // `spawn_signal_task` (TD Logic section): this is deliberately after
    // AdminState and both plane subscriptions exist, so SIGTERM/SIGINT flips
    // the same controller every startup participant already observes.
    tokio::spawn(crate::interfaces::admin::drain_on_shutdown_signal(
        drain.clone(),
        server_lifecycle::signal::wait_shutdown_signal(),
    ));
    let admin_listener = tokio::net::TcpListener::bind(plan.admin_bind.socket_addr()).await?;

    let listener = server_tcp::bind(&server_config).await?;
    println!("pgpool serve: listening on {}", listener.local_addr()?);
    println!(
        "pgpool serve: backend {}:{}",
        args.backend_host, args.backend_port
    );
    println!(
        "pgpool serve: admin plane on {}",
        admin_listener.local_addr()?
    );

    tokio::join!(
        server_tcp::serve(listener, server_config, handler, tcp_shutdown),
        server_http::serve_h2c_with_options(
            admin_listener,
            admin_router,
            plan.admin_h2c,
            admin_shutdown,
        ),
    );

    Ok(())
}
// </HANDWRITE>
// </HANDWRITE>
