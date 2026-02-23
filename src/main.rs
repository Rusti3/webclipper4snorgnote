use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use notebooklm_runner::app::{RunnerConfig, run_from_deeplink, run_once};

#[derive(Debug, Parser)]
#[command(name = "notebooklm_runner")]
#[command(about = "One-shot NotebookLM runner: start.txt -> end.txt")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),
    Deeplink(DeepLinkArgs),
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long, default_value = "start.txt")]
    input: PathBuf,
    #[arg(long, default_value = "end.txt")]
    output: PathBuf,
    #[arg(long, default_value = "Auto Notebook")]
    title: String,
    #[arg(long, default_value = "sidecar/bridge.js")]
    sidecar_script: PathBuf,
    #[arg(long, default_value = "node")]
    node_path: PathBuf,
    #[arg(long)]
    profile_dir: Option<PathBuf>,
    #[arg(long)]
    browser_path: Option<PathBuf>,
    #[arg(long, default_value_t = 600)]
    timeout_sec: u64,
}

#[derive(Debug, Args)]
struct DeepLinkArgs {
    uri: String,
    #[arg(long, default_value = "start.txt")]
    input: PathBuf,
    #[arg(long, default_value = "end.txt")]
    output: PathBuf,
    #[arg(long, default_value = "Auto Notebook")]
    title: String,
    #[arg(long, default_value = "sidecar/bridge.js")]
    sidecar_script: PathBuf,
    #[arg(long, default_value = "node")]
    node_path: PathBuf,
    #[arg(long)]
    profile_dir: Option<PathBuf>,
    #[arg(long)]
    browser_path: Option<PathBuf>,
    #[arg(long, default_value_t = 600)]
    timeout_sec: u64,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run(args) => run_once(&RunnerConfig {
            input: args.input,
            output: args.output,
            title: args.title,
            sidecar_script: args.sidecar_script,
            node_path: args.node_path,
            profile_dir: args.profile_dir,
            browser_path: args.browser_path,
            timeout_sec: args.timeout_sec,
        }),
        Commands::Deeplink(args) => run_from_deeplink(
            &args.uri,
            &RunnerConfig {
                input: args.input,
                output: args.output,
                title: args.title,
                sidecar_script: args.sidecar_script,
                node_path: args.node_path,
                profile_dir: args.profile_dir,
                browser_path: args.browser_path,
                timeout_sec: args.timeout_sec,
            },
        ),
    };

    if let Err(err) = result {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}
