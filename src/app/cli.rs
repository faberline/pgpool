use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "pgpool",
    version,
    about = "working-name PostgreSQL pooler service"
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
    /// Print the shared-runtime plan for the pooler data plane and admin plane.
    RuntimePlan,
    /// Print the offline admin API contract.
    Spec(SpecArgs),
    /// Print agent-facing LLM topics, offline.
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release.
    Upgrade(UpgradeArgs),
    /// Search, view, file, and comment on pgpool issues.
    Issue(IssueArgs),
    /// Run the session-mode PostgreSQL proxy: bind the frontend, dial the
    /// configured backend per client, and relay until drain/shutdown.
    Serve(ServeArgs),
    /// Render layered Kubernetes artifacts.
    K8s(K8sArgs),
}

#[derive(clap::Args)]
pub(super) struct SpecArgs {
    /// Contract format to print.
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    pub(super) format: SpecFormat,
}

#[derive(Clone, Copy, ValueEnum)]
pub(super) enum SpecFormat {
    Openapi,
    OpenapiYaml,
    JsonSchema,
    Routes,
}

#[derive(clap::Args)]
pub(super) struct LlmArgs {
    /// Topic: outline, workflow, api, or boundaries.
    #[arg(long, default_value = "outline")]
    pub(super) topic: String,
    /// Output format: md or json.
    #[arg(long, default_value = "md")]
    pub(super) format: String,
}

#[derive(clap::Args)]
pub(super) struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    pub(super) check: bool,
    /// Install this exact version (`0.4.3` or `pgpool@0.4.3`) instead of latest.
    #[arg(long = "version")]
    pub(super) tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    pub(super) force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub(super) yes: bool,
}

/// Config section of the session-mode-proxy TD: exact env/flag/default
/// surface for `pgpool serve`.
#[derive(clap::Args)]
pub(super) struct ServeArgs {
    /// Postgres backend host this session-mode proxy dials per client.
    #[arg(long, env = "PGPOOL_BACKEND_HOST", default_value = "127.0.0.1")]
    pub(super) backend_host: String,
    /// Postgres backend port.
    #[arg(long, env = "PGPOOL_BACKEND_PORT", default_value_t = 5432)]
    pub(super) backend_port: u16,
    /// Bound on the backend TCP connect, in milliseconds.
    #[arg(
        long,
        env = "PGPOOL_BACKEND_CONNECT_TIMEOUT_MS",
        default_value_t = 5000
    )]
    pub(super) backend_connect_timeout_ms: u64,
    /// Override the frontend bind address (`host:port`); defaults to the
    /// `RuntimePlan`'s frontend bind (`0.0.0.0:6432`) unchanged.
    #[arg(long, env = "PGPOOL_FRONTEND_BIND")]
    pub(super) bind: Option<String>,
    /// Grace window for in-flight sessions after SIGTERM/SIGINT, in
    /// milliseconds.
    #[arg(long, env = "PGPOOL_DRAIN_TIMEOUT_MS", default_value_t = 30000)]
    pub(super) drain_timeout_ms: u64,
    /// Bound on `BackendPool::acquire()`/`acquire_fresh()` waiting for an
    /// idle/freed backend slot before `PoolError::Saturated`, in
    /// milliseconds.
    #[arg(long, env = "PGPOOL_POOL_ACQUIRE_TIMEOUT_MS", default_value_t = 5000)]
    pub(super) pool_acquire_timeout_ms: u64,
    /// Per-Pod backend connection quota admitted by the control plane.
    #[arg(long, env = "PGPOOL_MAX_BACKEND_CONNECTIONS", default_value_t = 512)]
    pub(super) max_backend_connections: usize,
    /// Endpoint name used by the asynchronous reserve-lease worker. Empty
    /// disables reserve demand and keeps the historical local pool behavior.
    #[arg(long, env = "PGPOOL_RESERVE_ENDPOINT", default_value = "")]
    pub(super) reserve_endpoint: String,
    /// Stable Pod identity for the reserve grant. The Deployment renderer
    /// supplies this from metadata.name through the Downward API.
    #[arg(long, env = "PGPOOL_RESERVE_POD", default_value = "")]
    pub(super) reserve_pod: String,
    /// Stable pod identity used to identify every physical PostgreSQL backend
    /// connection opened by this pgpool process. The Deployment renderer
    /// supplies metadata.name through the Downward API.
    #[arg(long, env = "PGPOOL_POD_NAME", default_value = "local")]
    pub(super) pod_name: String,
    /// Normal-pool wait before a saturated transaction is allowed to signal
    /// a background reserve-grant request.
    #[arg(long, env = "PGPOOL_RESERVE_POOL_TIMEOUT_MS", default_value_t = 1000)]
    pub(super) reserve_pool_timeout_ms: u64,
    /// Terminal bounded wait for a normal or already granted reserve lease.
    #[arg(long, env = "PGPOOL_QUEUE_WAIT_TIMEOUT_MS", default_value_t = 5000)]
    pub(super) queue_wait_timeout_ms: u64,
    /// Idle reserve-grant release timeout, in milliseconds.
    #[arg(long, env = "PGPOOL_RESERVE_IDLE_TIMEOUT_MS", default_value_t = 30000)]
    pub(super) reserve_idle_timeout_ms: u64,
    /// Lease TTL requested by the background reserve worker, in seconds.
    #[arg(long, env = "PGPOOL_RESERVE_LEASE_TTL_SECONDS", default_value_t = 60)]
    pub(super) reserve_lease_ttl_seconds: u64,
    /// Maximum reserve units requested in one background allocator exchange.
    #[arg(long, env = "PGPOOL_RESERVE_REQUEST_CHUNK_SIZE", default_value_t = 1)]
    pub(super) reserve_request_chunk_size: u32,
    /// Override the admin plane bind address (`host:port`); defaults to the
    /// `RuntimePlan`'s admin bind (`0.0.0.0:9080`) unchanged.
    #[arg(long, env = "PGPOOL_ADMIN_BIND")]
    pub(super) admin_bind: Option<String>,
    /// Operator-facing name for this process's single backend pool; labels
    /// `PoolStats.name`, the `{pool}` path segment in
    /// `GET /pools/{pool}/stats`, and every `/metrics` gauge's `pool=`
    /// label.
    #[arg(long, env = "PGPOOL_POOL_NAME", default_value = "default")]
    pub(super) pool_name: String,
    /// Bound on the admin HTTP plane's own graceful shutdown once drain
    /// starts, in milliseconds.
    #[arg(long, env = "PGPOOL_ADMIN_DRAIN_TIMEOUT_MS", default_value_t = 30000)]
    pub(super) admin_drain_timeout_ms: u64,
}

#[derive(clap::Args)]
pub(super) struct K8sArgs {
    #[command(subcommand)]
    pub(super) command: K8sCommand,
}

#[derive(Subcommand)]
pub(super) enum K8sCommand {
    /// Render the cluster-scoped Pgpool CustomResourceDefinition.
    Crd(K8sCrdArgs),
    /// Render or run the Pgpool operator control plane.
    Operator(K8sOperatorArgs),
    /// Render app-namespace Pgpool instance artifacts.
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args)]
pub(super) struct K8sCrdArgs {
    #[command(subcommand)]
    pub(super) command: K8sCrdCommand,
}

#[derive(Subcommand)]
pub(super) enum K8sCrdCommand {
    /// Render the Pgpool CustomResourceDefinition YAML.
    Render(K8sOutputArgs),
}

#[derive(clap::Args)]
pub(super) struct K8sOperatorArgs {
    #[command(subcommand)]
    pub(super) command: K8sOperatorCommand,
}

#[derive(Subcommand)]
pub(super) enum K8sOperatorCommand {
    /// Render operator namespace, RBAC, and Deployment YAML.
    Render(K8sOperatorRenderArgs),
    /// Run the shared leader-elected reconcile controller.
    Run,
}

#[derive(clap::Args)]
pub(super) struct K8sOutputArgs {
    /// Write YAML to a path instead of stdout.
    #[arg(long)]
    pub(super) out: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
pub(super) struct K8sOperatorRenderArgs {
    /// Namespace containing the operator control plane.
    #[arg(long, default_value = "pgpool-system")]
    pub(super) namespace: String,
    /// Write YAML to a path instead of stdout.
    #[arg(long)]
    pub(super) out: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
pub(super) struct K8sInstanceArgs {
    #[command(subcommand)]
    pub(super) command: K8sInstanceCommand,
}

#[derive(Subcommand)]
pub(super) enum K8sInstanceCommand {
    /// Render the stateless Deployment, ClusterIP Service, ServiceAccount, and PDB.
    Render(K8sInstanceRenderArgs),
}

#[derive(clap::Args)]
pub(super) struct K8sInstanceRenderArgs {
    #[arg(long, value_enum, default_value_t = K8sProfile::Dev)]
    pub(super) profile: K8sProfile,
    /// Write YAML to a path instead of stdout.
    #[arg(long)]
    pub(super) out: Option<std::path::PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum K8sProfile {
    Dev,
    Staging,
    Prod,
    Template,
}

impl From<K8sProfile> for crate::application::k8s::InstanceProfile {
    fn from(value: K8sProfile) -> Self {
        match value {
            K8sProfile::Dev => Self::Dev,
            K8sProfile::Staging => Self::Staging,
            K8sProfile::Prod => Self::Prod,
            K8sProfile::Template => Self::Template,
        }
    }
}

#[derive(clap::Args)]
pub(super) struct IssueArgs {
    #[command(subcommand)]
    pub(super) command: IssueCommand,
}

#[derive(Subcommand)]
pub(super) enum IssueCommand {
    /// Search pgpool issues (`project:pgpool`); omit query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich pgpool issue.
    Create(IssueCreateArgs),
    /// Comment on an issue and ensure it is open.
    Comment(IssueCommentArgs),
}

#[derive(clap::Args)]
pub(super) struct IssueSearchArgs {
    #[arg(value_name = "QUERY", num_args = 0..)]
    pub(super) query: Vec<String>,
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    pub(super) state: String,
    #[arg(long, default_value_t = 20)]
    pub(super) limit: u32,
}

#[derive(clap::Args)]
pub(super) struct IssueViewArgs {
    pub(super) number: u64,
}

#[derive(clap::Args)]
pub(super) struct IssueCreateArgs {
    #[arg(short = 't', long)]
    pub(super) title: Option<String>,
    #[arg(value_name = "MSG", num_args = 0..)]
    pub(super) message: Vec<String>,
    #[arg(long)]
    pub(super) url: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) label: Vec<String>,
    #[arg(long)]
    pub(super) dry_run: bool,
    #[arg(short = 'y', long)]
    pub(super) yes: bool,
}

#[derive(clap::Args)]
pub(super) struct IssueCommentArgs {
    pub(super) number: u64,
    #[arg(value_name = "MSG", num_args = 0..)]
    pub(super) message: Vec<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) dry_run: bool,
    #[arg(short = 'y', long)]
    pub(super) yes: bool,
}
