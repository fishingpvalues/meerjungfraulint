use clap::{Parser, Subcommand};
use meerjungfraulint::cli;

#[derive(Parser)]
#[command(name = "meerjungfraulint")]
#[command(about = "A fast linter for Markdown files with Mermaid diagrams")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint files or directories
    Lint {
        /// Path to file or directory to lint
        path: String,
        /// Output format: text, json, sarif
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Auto-fix safe issues
        #[arg(long)]
        fix: bool,
    },
    // /// Start LSP server
    // Lsp,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint { path, format, fix } => {
            cli::run_lint(&path, &format, fix).await?;
        }
        // Commands::Lsp => {
        //     cli::run_lsp().await?;
        // }
    }

    Ok(())
}
