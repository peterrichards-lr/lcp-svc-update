mod cli;
mod core;
mod utils;

use crate::cli::{App, AppCommands};
use crate::core::{LcpJson, LiferayProject, ProjectType, Workspace};
use crate::utils::{git, scraper};
use clap::Parser;
use std::collections::HashMap;
use std::path::Path;

fn find_matching_image(
    service_id: &str,
    current_image: &str,
    latest_versions: &HashMap<String, String>,
    detected_version: Option<&str>,
) -> Option<String> {
    if service_id == "liferay" {
        // 1. Try detected version from gradle.properties
        if let Some(v) = detected_version {
            let specific_key = format!("liferay:{}", v);
            if let Some(latest) = latest_versions.get(&specific_key) {
                return Some(latest.clone());
            }
        }

        // 2. Fallback to matching major.minor version from current image tag
        if let Some(version_part) = current_image.split(':').nth(1) {
            if let Some(v) = version_part.split('-').next() {
                let specific_key = format!("liferay:{}", v);
                if let Some(latest) = latest_versions.get(&specific_key) {
                    return Some(latest.clone());
                }
            }
        }
    }

    // Fallback to direct ID match
    latest_versions.get(service_id).cloned()
}

fn git_commit_and_push(root: &Path, updates: &[(String, String, String)]) -> anyhow::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }

    println!("\nCommitting changes...");

    // 1. Git Add
    let paths: Vec<std::path::PathBuf> = updates
        .iter()
        .map(|(id, _, _)| root.join(id).join("LCP.json"))
        .collect();

    let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    git::git_add(root, &path_refs)?;

    // 2. Git Commit
    let summary = "build: update Liferay Cloud service images";
    let mut description =
        String::from("Updated the following service images to their latest versions:\n");
    for (id, old, new) in updates {
        description.push_str(&format!("\n- {}: {} -> {}", id, old, new));
    }

    let commit_msg = format!("{}\n\n{}", summary, description);
    git::git_commit(root, &commit_msg)?;

    // 3. Git Push
    println!("Pushing to remote...");
    git::git_push(root)?;

    println!("Successfully committed and pushed updates.");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    match args.command {
        AppCommands::Check { workspace } => {
            let ws = LiferayProject {
                current_dir: workspace.canonicalize().unwrap_or(workspace),
            };

            println!("Fetching latest service versions from Liferay Support...");
            let latest_link = scraper::fetch_latest_update_link()?;
            let latest_versions = scraper::fetch_service_versions(&latest_link)?;

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

            let services = ws.find_services(&root)?;

            if services.is_empty() {
                println!("\nNo services with LCP.json found in: {:?}", root);
            } else {
                for service_path in services {
                    let lcp_path = service_path.join("LCP.json");
                    if let Ok(lcp) = LcpJson::load(&lcp_path) {
                        let service_id = lcp.id.clone().unwrap_or_default();
                        let current_image = lcp.image.clone().unwrap_or_default();

                        if let Some(latest_image) = find_matching_image(
                            &service_id,
                            &current_image,
                            &latest_versions,
                            detected_liferay_v.as_deref(),
                        ) {
                            if current_image != latest_image {
                                println!("  [UPDATE AVAILABLE] Service: {}", service_id);
                                println!("    Current: {}", current_image);
                                println!("    Latest:  {}", latest_image);
                            } else {
                                println!("  [OK] Service: {} is up to date.", service_id);
                            }
                        } else {
                            println!(
                                "  [SKIP] Service: {} (No latest version found for this ID)",
                                service_id
                            );
                        }
                    }
                }
            }
        }
        AppCommands::Apply {
            workspace,
            dry_run,
            commit,
        } => {
            let ws = LiferayProject {
                current_dir: workspace.canonicalize().unwrap_or(workspace),
            };
            let root = ws.find_root()?;
            let detected_liferay_v = ws.get_liferay_version(&root);

            let services = ws.find_services(&root)?;

            println!("Fetching latest service versions from Liferay Support...");
            let latest_link = scraper::fetch_latest_update_link()?;
            let latest_versions = scraper::fetch_service_versions(&latest_link)?;

            if services.is_empty() {
                println!("No services found to update.");
                return Ok(());
            }

            let mut updated_services = Vec::new();

            for service_path in services {
                let lcp_path = service_path.join("LCP.json");
                if let Ok(mut lcp) = LcpJson::load(&lcp_path) {
                    let service_id = lcp.id.clone().unwrap_or_default();
                    let current_image = lcp.image.clone().unwrap_or_default();

                    if let Some(latest_image) = find_matching_image(
                        &service_id,
                        &current_image,
                        &latest_versions,
                        detected_liferay_v.as_deref(),
                    ) {
                        if current_image != latest_image {
                            if dry_run {
                                println!(
                                    "[DRY RUN] Would update {}: {} -> {}",
                                    service_id, current_image, latest_image
                                );
                            } else {
                                println!("Updating service: {} -> {}", service_id, latest_image);
                                lcp.update_image(&latest_image);
                                lcp.save(&lcp_path)?;
                                updated_services.push((service_id, current_image, latest_image));
                            }
                        }
                    }
                }
            }

            if commit && !dry_run && !updated_services.is_empty() {
                git_commit_and_push(&root, &updated_services)?;
            }
        }
    }

    Ok(())
}
