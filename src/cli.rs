use crate::utils::scraper::DEFAULT_CHANGELOG_URL;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lcp-svc-update")]
#[command(version, about = "Liferay Services Updater - Update Liferay Cloud service Docker image references (LCP.json)", long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: AppCommands,
}

#[derive(Subcommand)]
pub enum AppCommands {
    /// Check for the latest service image versions published by Liferay
    Check {
        /// Path to the Liferay Cloud workspace (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        workspace: PathBuf,

        #[command(flatten)]
        source: SourceArgs,
    },
    /// Apply the latest service image versions to LCP.json files in the workspace
    Apply {
        /// Path to the Liferay Cloud workspace (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        workspace: PathBuf,

        /// Only show what would be updated without actually writing to files
        #[arg(long)]
        dry_run: bool,

        /// Commit the updated LCP.json files
        #[arg(short, long)]
        commit: bool,

        /// Push the commit to the remote repository (requires --commit)
        #[arg(short, long, requires = "commit")]
        push: bool,

        #[command(flatten)]
        source: SourceArgs,
    },
}

/// Where the latest service images are read from. Both can be overridden
/// without rebuilding, either on the command line or via the environment.
#[derive(Args)]
pub struct SourceArgs {
    /// Services changelog index used to discover the latest release notes
    #[arg(long, env = "LCP_SVC_UPDATE_CHANGELOG_URL", default_value = DEFAULT_CHANGELOG_URL)]
    pub changelog_url: String,

    /// Read images from this release notes page instead of discovering the latest one
    #[arg(long, env = "LCP_SVC_UPDATE_RELEASE_URL")]
    pub release_url: Option<String>,
}
