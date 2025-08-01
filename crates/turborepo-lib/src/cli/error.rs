use std::backtrace;

use itertools::Itertools;
use miette::Diagnostic;
use thiserror::Error;
use turborepo_repository::package_graph;
use turborepo_signals::{listeners::get_signal, SignalHandler};
use turborepo_telemetry::events::command::CommandEventBuilder;
use turborepo_ui::{color, BOLD, GREY};

use crate::{
    commands::{bin, generate, link, login, ls, prune, CommandBase},
    daemon::DaemonError,
    query,
    rewrite_json::RewriteError,
    run,
    run::{builder::RunBuilder, watch},
};

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("No command specified.")]
    NoCommand(#[backtrace] backtrace::Backtrace),
    #[error("{0}")]
    Bin(#[from] bin::Error, #[backtrace] backtrace::Backtrace),
    #[error(transparent)]
    Boundaries(#[from] crate::boundaries::Error),
    #[error(transparent)]
    Clone(#[from] crate::commands::clone::Error),
    #[error(transparent)]
    Path(#[from] turbopath::PathError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Config(#[from] crate::config::Error),
    #[error(transparent)]
    ChromeTracing(#[from] crate::tracing::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    BuildPackageGraph(#[from] package_graph::builder::Error),
    #[error(transparent)]
    Rewrite(#[from] RewriteError),
    #[error(transparent)]
    Auth(#[from] turborepo_auth::Error),
    #[error(transparent)]
    Daemon(#[from] DaemonError),
    #[error(transparent)]
    Generate(#[from] generate::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Ls(#[from] ls::Error),
    #[error(transparent)]
    Login(#[from] login::Error),
    #[error(transparent)]
    Link(#[from] link::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Prune(#[from] prune::Error),
    #[error(transparent)]
    PackageJson(#[from] turborepo_repository::package_json::Error),
    #[error(transparent)]
    PackageManager(#[from] turborepo_repository::package_manager::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Run(#[from] run::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Query(#[from] query::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Watch(#[from] watch::Error),
    #[error(transparent)]
    Opts(#[from] crate::opts::Error),
    #[error(transparent)]
    SignalListener(#[from] turborepo_signals::listeners::Error),
    #[error(transparent)]
    Dialoguer(#[from] dialoguer::Error),
}

const MAX_CHARS_PER_TASK_LINE: usize = 100;

pub async fn print_potential_tasks(
    base: CommandBase,
    telemetry: CommandEventBuilder,
) -> Result<(), Error> {
    let signal = get_signal()?;
    let handler = SignalHandler::new(signal);
    let color_config = base.color_config;

    let run_builder = RunBuilder::new(base)?;
    let run = run_builder.build(&handler, telemetry).await?;
    let potential_tasks = run.get_potential_tasks()?;

    println!("No tasks provided, here are some potential ones\n",);

    for (task, packages) in potential_tasks
        .into_iter()
        .sorted_by(|(_, a), (_, b)| b.len().cmp(&a.len()))
    {
        let task = color!(color_config, BOLD, "{}", task);
        let mut line_length = 0;

        let mut packages_str = String::with_capacity(MAX_CHARS_PER_TASK_LINE);
        for (idx, package) in packages.iter().sorted().enumerate() {
            if line_length > MAX_CHARS_PER_TASK_LINE {
                if idx != packages.len() {
                    packages_str.push_str(&format!(" and {} more", packages.len() - idx));
                }

                break;
            }

            line_length += package.len() + 2;
            if idx != 0 {
                packages_str.push_str(", ");
            }
            packages_str.push_str(package);
        }

        let packages = color!(color_config, GREY, "{}", packages_str);

        println!("  {task}\n    {packages}")
    }

    Ok(())
}
