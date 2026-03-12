use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lcp-svc-update")]
#[command(version, about = "Update Liferay Cloud service Docker image references (LCP.json)", long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: AppCommands,
}

#[derive(Subcommand)]
pub enum AppCommands {
    /// Check for the latest service image versions on Liferay Support site
    Check {
        /// Path to the Liferay Cloud workspace (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        workspace: PathBuf,
    },
    /// Apply the latest service image versions to LCP.json files in the workspace
    Apply {
        /// Path to the Liferay Cloud workspace (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        workspace: PathBuf,

        /// Only show what would be updated without actually writing to files
        #[arg(long)]
        dry_run: bool,

        /// Commit and push changes to the remote repository
        #[arg(short, long)]
        commit: bool,
    },
}
