use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use notebooklm_runner::app::{AppConfig, run_from_new_deeplink};
use notebooklm_runner::protocol::{
    ProtocolRegistrationStatus, ensure_protocol_registered, protocol_command_value,
};

#[derive(Debug, Parser)]
#[command(name = "notebooklm_runner")]
#[command(about = "Snorgnote clip receiver: deep-link data -> notes/*.md")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Deeplink(DeepLinkArgs),
    InstallProtocol,
}

#[derive(Debug, Args)]
struct CommonArgs {
    #[arg(long, default_value = "notes")]
    notes_dir: PathBuf,
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
    let registration_result = auto_register_protocol();
    if let Err(err) = &registration_result {
        eprintln!("Warning: protocol auto-registration failed: {err:#}");
    }
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Deeplink(args)) => {
            let cfg = to_config(args.common);
            match run_from_new_deeplink(&args.uri, &cfg) {
                Ok(outcome) => {
                    println!("Saved note: {}", outcome.note_path.display());
                    println!("GeneratedClipId: {}", outcome.clip_id);
                    if outcome.clipped {
                        eprintln!("Warning: content was clipped due to payload size limit.");
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        Some(Commands::InstallProtocol) => registration_result,
        None => {
            let registration = registration_result;
            if registration.is_ok() {
                println!("Protocol is ready. You can close this window.");
            }
            registration
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
        timeout_sec: args.timeout_sec,
    }
}

fn auto_register_protocol() -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;
    let status = ensure_protocol_registered("snorgnote", &exe_path)?;
    match status {
        ProtocolRegistrationStatus::AlreadyRegistered => {}
        ProtocolRegistrationStatus::Updated => {
            println!(
                "Protocol registered: snorgnote:// -> {}",
                protocol_command_value(&exe_path)
            );
        }
        ProtocolRegistrationStatus::Skipped => {}
    }
    Ok(())
}
