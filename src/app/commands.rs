use anyhow::Result;
use clap::Parser;

use crate::interfaces::spec;

use super::cli::*;
use super::serve::serve;

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "pgpool",
    repo: "faberline/pgpool",
    target: env!("PGPOOL_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("PGPOOL_GIT_SHA"),
    built_at: env!("PGPOOL_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "working-name app scaffold and pooler rollout boundaries",
        body: spec::llm_workflow_md(),
    },
    cli_std::llm::Topic {
        id: "api",
        summary: "PostgreSQL frontend and admin API route inventory",
        body: spec::llm_api_md(),
    },
    cli_std::llm::Topic {
        id: "boundaries",
        summary: "platform adapter and shared runtime boundaries",
        body: spec::llm_boundaries_md(),
    },
];

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::RuntimePlan => runtime_plan(),
        Command::Spec(args) => spec(args),
        Command::Llm(args) => llm(args),
        Command::Upgrade(args) => {
            cli_std::upgrade::run(
                &TOOL,
                cli_std::upgrade::Options {
                    check: args.check,
                    tag: args.tag,
                    force: args.force,
                    yes: args.yes,
                },
            )
            .await
        }
        Command::Issue(args) => issue(args).await,
        Command::Serve(args) => serve(args).await,
        Command::K8s(args) => k8s(args).await,
    }
}

async fn k8s(args: K8sArgs) -> Result<()> {
    match args.command {
        K8sCommand::Crd(args) => match args.command {
            K8sCrdCommand::Render(args) => {
                write_or_print(args.out, crate::interfaces::operator::crd_yaml())
            }
        },
        K8sCommand::Operator(args) => match args.command {
            K8sOperatorCommand::Render(args) => write_or_print(
                args.out,
                crate::interfaces::operator::operator_yaml(&args.namespace),
            ),
            K8sOperatorCommand::Run => crate::interfaces::operator::run().await,
        },
        K8sCommand::Instance(args) => match args.command {
            K8sInstanceCommand::Render(args) => write_or_print(
                args.out,
                crate::interfaces::operator::instance_yaml(args.profile.into()),
            ),
        },
    }
}

fn write_or_print(path: Option<std::path::PathBuf>, yaml: String) -> Result<()> {
    if let Some(path) = path {
        std::fs::write(&path, yaml)?;
        println!("wrote {}", path.display());
    } else {
        print!("{yaml}");
    }
    Ok(())
}

fn runtime_plan() -> Result<()> {
    println!("{}", crate::application::runtime_plan::runtime_plan_json());
    println!("next: pgpool spec --format routes");
    Ok(())
}

fn spec(args: SpecArgs) -> Result<()> {
    let out = match args.format {
        SpecFormat::Openapi => spec::openapi_json(),
        SpecFormat::OpenapiYaml => spec::openapi_yaml(),
        SpecFormat::JsonSchema => spec::json_schema_json(),
        SpecFormat::Routes => spec::routes_json(),
    };
    println!("{out}");
    Ok(())
}

fn llm(args: LlmArgs) -> Result<()> {
    let out = cli_std::llm::render(
        TOOL.project,
        TOOL.version,
        LLM_TOPICS,
        &args.topic,
        cli_std::llm::Format::parse(&args.format),
    )?;
    println!("{out}");
    Ok(())
}

async fn issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search(args) => {
            let query = (!args.query.is_empty()).then(|| args.query.join(" "));
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query,
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await
        }
        IssueCommand::View(args) => cli_std::issue::view(&TOOL, args.number).await,
        IssueCommand::Create(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            let title = args.title.unwrap_or_else(|| {
                message
                    .as_deref()
                    .and_then(|msg| msg.lines().next())
                    .map(|head| format!("pgpool: {}", head.chars().take(72).collect::<String>()))
                    .unwrap_or_else(|| "pgpool: issue report".to_string())
            });
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: args.url,
                    repo: args.repo,
                    label: std::iter::once("project:pgpool".to_string())
                        .chain(args.label)
                        .collect(),
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
        IssueCommand::Comment(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            cli_std::issue::comment(
                &TOOL,
                cli_std::issue::CommentOptions {
                    number: args.number,
                    message,
                    repo: args.repo,
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
    }
}
