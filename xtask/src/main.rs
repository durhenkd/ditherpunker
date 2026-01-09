use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for ditherpunker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenerateMatrices,
    Ci,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateMatrices => generate_matrices(),
        Commands::Ci => ci(),
    }
}

fn generate_matrices() -> Result<()> {
    // TODO: Implement matrix generation logic
    // This could generate Bayer matrices programmatically and save as textures for local dev
    // or process Blue Noise patterns from source data
    //
    // This can also be used to pack a core set of texture and bundle it together with a possible
    // release
    Ok(())
}

/// WIP: just an example, not a requirement, subject to change
///
/// can run benches, tests, bundle reports and so on...
fn ci() -> Result<()> {
    run_command("cargo", &["fmt", "--all", "--check"])?;
    run_command(
        "cargo",
        &[
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
            "-A",
            "clippy::needless_range_loop",
        ],
    )?;
    run_command("cargo", &["build", "--all-features"])?;
    run_command("cargo", &["test", "--all-features"])?;
    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    use std::process::Command;
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        anyhow::bail!("Command failed: {} {}", cmd, args.join(" "));
    }
    Ok(())
}
