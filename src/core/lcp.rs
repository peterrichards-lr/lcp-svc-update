use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LcpJson {
    pub id: Option<String>,
    pub image: Option<String>,
    pub kind: Option<String>,
    #[serde(flatten)]
    pub other_fields: HashMap<String, serde_json::Value>,
}

impl LcpJson {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let lcp: LcpJson = serde_json::from_str(&content)?;
        Ok(lcp)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn update_image(&mut self, new_image: &str) {
        self.image = Some(new_image.to_string());
    }
}
