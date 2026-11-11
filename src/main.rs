mod cli;
mod core;
mod utils;

use crate::cli::{App, AppCommands, SourceArgs};
use crate::core::{LcpJson, LiferayProject, ProjectType, ServiceImages, Workspace};
use crate::utils::{git, scraper};
use clap::Parser;
use std::path::{Path, PathBuf};

struct Update {
    path: PathBuf,
    service_id: String,
    from: String,
    to: String,
}

fn fetch_latest_images(source: &SourceArgs) -> anyhow::Result<ServiceImages> {
    let client = scraper::client()?;
    let release_url = match &source.release_url {
        Some(url) => url.clone(),
        None => {
            println!("Finding latest service release notes...");
            scraper::find_latest_release_url(&client, &source.changelog_url)?
        }
    };
    println!("Reading service images from: {}", release_url);
    scraper::fetch_service_images(&client, &release_url)
}

/// Loads the LCP.json of every service in the workspace, reporting any that can't be read.
fn load_services(ws: &LiferayProject, root: &Path) -> anyhow::Result<Vec<(PathBuf, LcpJson)>> {
    let mut services = Vec::new();
    for service_path in ws.find_services(root)? {
        let lcp_path = service_path.join("LCP.json");
        match LcpJson::load(&lcp_path) {
            Ok(lcp) => services.push((lcp_path, lcp)),
            Err(e) => eprintln!("  [WARN] Could not read {}: {}", lcp_path.display(), e),
        }
    }
    services.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(services)
}

fn git_commit_and_push(root: &Path, updates: &[Update], push: bool) -> anyhow::Result<()> {
    println!("\nCommitting changes...");

    let paths: Vec<&Path> = updates.iter().map(|u| u.path.as_path()).collect();
    git::git_add(root, &paths)?;

    let summary = "build: update Liferay Cloud service images";
    let mut description =
        String::from("Updated the following service images to their latest versions:\n");
    for update in updates {
        description.push_str(&format!(
            "\n- {}: {} -> {}",
            update.service_id, update.from, update.to
        ));
    }

    let commit_msg = format!("{}\n\n{}", summary, description);
    git::git_commit(root, &commit_msg, &paths)?;

    if push {
        println!("Pushing to remote...");
        git::git_push(root)?;
        println!("Successfully committed and pushed updates.");
    } else {
        println!("Successfully committed updates.");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    match args.command {
        AppCommands::Check { workspace, source } => {
            let ws = LiferayProject {
                current_dir: workspace.canonicalize().unwrap_or(workspace),
            };

            let latest_images = fetch_latest_images(&source)?;

            let root = ws.find_root()?;
            let project_type = ws.detect_type(&root);
            let detected_liferay_v = ws.get_liferay_version(&root);

            println!("Checking workspace at: {:?}", root);

            let project_desc = match project_type {
                ProjectType::LiferayWorkspace => "Liferay Workspace (Traditional)",
                ProjectType::LiferayCloud => "Liferay Cloud (LXC/DXP Cloud)",
                ProjectType::ClientExtension => "Liferay Client Extension",
                ProjectType::Unknown => "Unknown project structure",
            };
            println!("Project type: {}", project_desc);

            if let Some(ref v) = detected_liferay_v {
                println!("Detected Liferay version: {}", v);
            }

            let services = load_services(&ws, &root)?;

            if services.is_empty() {
                println!("\nNo services with LCP.json found in: {:?}", root);
            }

            for (_, lcp) in services {
                let service_id = lcp.id.unwrap_or_default();
                let current_image = lcp.image.unwrap_or_default();

                match latest_images.find_match(
                    &service_id,
                    &current_image,
                    detected_liferay_v.as_deref(),
                ) {
                    Some(latest_image) if latest_image != current_image => {
                        println!("  [UPDATE AVAILABLE] Service: {}", service_id);
                        println!("    Current: {}", current_image);
                        println!("    Latest:  {}", latest_image);
                    }
                    Some(_) => println!("  [OK] Service: {} is up to date.", service_id),
                    None => println!(
                        "  [SKIP] Service: {} (No latest version found for this ID)",
                        service_id
                    ),
                }
            }
        }
        AppCommands::Apply {
            workspace,
            dry_run,
            commit,
            push,
            source,
        } => {
            let ws = LiferayProject {
                current_dir: workspace.canonicalize().unwrap_or(workspace),
            };
            let root = ws.find_root()?;
            let detected_liferay_v = ws.get_liferay_version(&root);

            let services = load_services(&ws, &root)?;
            if services.is_empty() {
                println!("No services found to update.");
                return Ok(());
            }

            let latest_images = fetch_latest_images(&source)?;

            let mut pending = Vec::new();
            for (path, lcp) in services {
                let service_id = lcp.id.clone().unwrap_or_default();
                let current_image = lcp.image.clone().unwrap_or_default();

                if let Some(latest_image) = latest_images.find_match(
                    &service_id,
                    &current_image,
                    detected_liferay_v.as_deref(),
                ) {
                    if latest_image != current_image {
                        let update = Update {
                            path,
                            service_id,
                            from: current_image,
                            to: latest_image.to_string(),
                        };
                        pending.push((update, lcp));
                    }
                }
            }

            if pending.is_empty() {
                println!("All services are up to date.");
                return Ok(());
            }

            if dry_run {
                for (update, _) in &pending {
                    println!(
                        "[DRY RUN] Would update {}: {} -> {}",
                        update.service_id, update.from, update.to
                    );
                }
                return Ok(());
            }

            // Check before writing anything, so a refused commit leaves the workspace untouched
            if commit {
                if !git::is_repository(&root) {
                    anyhow::bail!("Cannot commit: {:?} is not inside a Git repository", root);
                }
                let paths: Vec<&Path> = pending.iter().map(|(u, _)| u.path.as_path()).collect();
                let changes = git::uncommitted_changes(&root, &paths)?;
                if !changes.is_empty() {
                    anyhow::bail!(
                        "Cannot commit: these LCP.json files already have uncommitted changes:\n{}",
                        changes.join("\n")
                    );
                }
            }

            let mut updated_services = Vec::new();
            for (update, mut lcp) in pending {
                println!("Updating service: {} -> {}", update.service_id, update.to);
                lcp.update_image(&update.to)?;
                lcp.save(&update.path)?;
                updated_services.push(update);
            }

            if commit {
                git_commit_and_push(&root, &updated_services, push)?;
            }
        }
    }

    Ok(())
}
