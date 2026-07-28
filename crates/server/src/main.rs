use std::process::exit;

use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use r2s_config::GlobalConfig;
use r2s_engine::Engine;
use r2s_server::{R2S_VERSION, down, greet, up};
use serde::Deserialize;
use tokio::io::AsyncReadExt;

/// Clap arg definition.
#[derive(Parser, Debug)]
#[command(
  author = "Reverier-Xu <reverier.xu@woooo.tech>",
  version,
  about = "Rhythm Arena Tournament API Platform",
  long_about = r#"
Rhythm Arena Tournament API Platform

Rhythm Arena is released under the Ret2Shell Public License 2.0,
a GPL-3.0-derived copyleft license with limited user-facing
monetization restrictions.

If you have any problems, please contact tech support <support@ret.sh.cn>.
    "#
)]
struct CliArgs {
  #[command(subcommand)]
  command: Option<Commands>,
}

/// Clap subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
  /// Run the server.
  Up,
  /// Remove all data and drop database, NEVER USE IT AT PRODUCTION
  /// ENVIRONMENT.
  Down,
  #[command(hide = true)]
  Internal(InternalArgs),
}

#[derive(clap::Args, Debug)]
struct InternalArgs {
  #[command(subcommand)]
  command: InternalCommands,
}

#[derive(Subcommand, Debug)]
enum InternalCommands {
  Score,
}

#[derive(Deserialize)]
struct ScoreRequest {
  source: String,
  context: String,
}

/// Server entry.
#[tokio::main]
async fn main() {
  let command = CliArgs::parse().command;
  if let Some(Commands::Internal(args)) = command {
    if let Err(error) = run_internal(args).await {
      eprintln!("{error}");
      exit(1);
    }
    return;
  }

  let config = match GlobalConfig::load() {
    Ok(config) => config,
    Err(e) => {
      eprintln!(
        "{}",
        "Rhythm Arena Tournament API Platform failed to init!"
          .red()
          .bold()
      );
      eprintln!("Version: {R2S_VERSION}");
      eprintln!("{}: {e}", "Failed to load server config".red().bold());
      eprintln!("Please check your configuration file and try again.");
      eprintln!(
        "If you are still suffering from this problem and don't know how to fix it, please contact tech support <support@ret.sh.cn>."
      );
      exit(1)
    }
  };
  greet();
  match match command {
    Some(Commands::Up) => up(config).await,
    Some(Commands::Down) => down(config).await,
    Some(Commands::Internal(_)) => unreachable!(),
    None => up(config).await,
  } {
    Ok(_) => {}
    Err(e) => {
      eprintln!(
        "{}",
        "Rhythm Arena Tournament API Platform failed to start!"
          .red()
          .bold()
      );
      eprintln!("Version: {R2S_VERSION}");
      eprintln!("{}: {e}", "Failed to start server".red().bold());
      eprintln!("Please check your configuration file and try again.");
      eprintln!(
        "If you are still suffering from this problem and don't know how to fix it, please contact tech support <support@ret.sh.cn>."
      );
      exit(1)
    }
  }
}

async fn run_internal(args: InternalArgs) -> anyhow::Result<()> {
  match args.command {
    InternalCommands::Score => {
      let mut input = String::new();
      tokio::io::stdin().read_to_string(&mut input).await?;
      let request: ScoreRequest = serde_json::from_str(&input)?;
      let output =
        Engine::execute_pure_json_limited(request.source, request.context, 64 * 1024 * 1024)
          .await?;
      println!("{output}");
      Ok(())
    }
  }
}
