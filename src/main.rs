use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use notebooklm_runner::app::{AppConfig, check_helper_health, run_from_new_deeplink};

#[derive(Debug, Parser)]
#[command(name = "notebooklm_runner")]
#[command(about = "Snorgnote clip receiver: deep-link -> helper -> notes/*.md")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Deeplink(DeepLinkArgs),
    HelperHealth(CommonArgs),
}

#[derive(Debug, Args)]
struct CommonArgs {
    #[arg(long, default_value = "notes")]
    notes_dir: PathBuf,
    #[arg(long, default_value = "http://127.0.0.1:27124")]
    helper_base_url: String,
    #[arg(long, default_value_t = 15)]
    timeout_sec: u64,
}

#[derive(Debug, Args)]
struct DeepLinkArgs {
    uri: String,
    #[command(flatten)]
    common: CommonArgs,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Deeplink(args) => {
            let cfg = to_config(args.common);
            match run_from_new_deeplink(&args.uri, &cfg) {
                Ok(outcome) => {
                    println!("Saved note: {}", outcome.note_path.display());
                    println!("ClipId: {}", outcome.clip_id);
                    if let Some(delete_error) = outcome.delete_error {
                        eprintln!(
                            "Warning: clip was saved, but helper cleanup failed: {delete_error}"
                        );
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        Commands::HelperHealth(args) => {
            let cfg = to_config(args);
            match check_helper_health(&cfg) {
                Ok(health) => {
                    println!("Helper OK: {}", health.ok);
                    if let Some(clips) = health.clips_in_memory {
                        println!("ClipsInMemory: {clips}");
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
    };

    if let Err(err) = result {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn to_config(args: CommonArgs) -> AppConfig {
    AppConfig {
        notes_dir: args.notes_dir,
        helper_base_url: args.helper_base_url,
        timeout_sec: args.timeout_sec,
    }
}
