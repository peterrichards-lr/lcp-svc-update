use std::fs;
use std::path::{Path, PathBuf};

pub trait Workspace {
    fn find_root(&self) -> anyhow::Result<PathBuf>;
    fn find_services(&self, root: &Path) -> anyhow::Result<Vec<PathBuf>>;
    fn get_liferay_version(&self, root: &Path) -> Option<String>;
}

pub struct LiferayCloudWorkspace {
    pub current_dir: PathBuf,
}

impl Workspace for LiferayCloudWorkspace {
    fn find_root(&self) -> anyhow::Result<PathBuf> {
        let mut path = self.current_dir.clone();
        loop {
            let has_liferay = path.join("liferay").exists();
            let has_webserver = path.join("webserver").exists();
            let has_backup = path.join("backup").exists();
            
            if has_liferay || has_webserver || has_backup {
                return Ok(path);
            }

            if !path.pop() {
                break;
            }
        }
        if self.current_dir.join("LCP.json").exists() {
            if let Some(parent) = self.current_dir.parent() {
                return Ok(parent.to_path_buf());
            }
        }
        
        Ok(self.current_dir.clone())
    }

    fn find_services(&self, root: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut services = Vec::new();
        let entries = fs::read_dir(root)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("LCP.json").exists() {
                services.push(path);
            }
        }

        Ok(services)
    }

    fn get_liferay_version(&self, root: &Path) -> Option<String> {
        let gradle_props_path = root.join("liferay").join("gradle.properties");
        if !gradle_props_path.exists() {
            return None;
        }

        let content = fs::read_to_string(gradle_props_path).ok()?;
        for line in content.lines() {
            if line.starts_with("liferay.workspace.product=") {
                let product = line.split('=').nth(1)?.trim();
                
                // Mapping logic
                if product.starts_with("dxp-202") || product.contains("7.4") {
                    return Some("7.4".to_string());
                } else if product.contains("7.3") {
                    return Some("7.3".to_string());
                } else if product.contains("7.2") {
                    return Some("7.2".to_string());
                } else if product.contains("7.1") {
                    return Some("7.1".to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_services() {
        let dir = tempdir().unwrap();
        let liferay_path = dir.path().join("liferay");
        let webserver_path = dir.path().join("webserver");

        fs::create_dir_all(&liferay_path).unwrap();
        fs::create_dir_all(&webserver_path).unwrap();
        fs::write(liferay_path.join("LCP.json"), "{}").unwrap();
        fs::write(webserver_path.join("LCP.json"), "{}").unwrap();

        let ws = LiferayCloudWorkspace {
            current_dir: dir.path().to_path_buf(),
        };

        let services = ws.find_services(dir.path()).unwrap();
        assert_eq!(services.len(), 2);
    }
}
