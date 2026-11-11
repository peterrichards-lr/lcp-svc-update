use anyhow::Context;
use std::fs;
use std::ops::Range;
use std::path::Path;

/// An LCP.json file. Only the top-level `image` value is ever rewritten; every
/// other byte (key order, indentation, trailing newline) is left untouched.
#[derive(Debug, Clone)]
pub struct LcpJson {
    content: String,
    pub id: Option<String>,
    pub image: Option<String>,
}

impl LcpJson {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(content)
    }

    fn parse(content: String) -> anyhow::Result<Self> {
        let value: serde_json::Value = serde_json::from_str(&content)?;
        let object = value
            .as_object()
            .context("LCP.json must contain a JSON object")?;
        let field = |name: &str| object.get(name).and_then(|v| v.as_str()).map(String::from);

        Ok(LcpJson {
            id: field("id"),
            image: field("image"),
            content,
        })
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, &self.content)?;
        Ok(())
    }

    pub fn update_image(&mut self, new_image: &str) -> anyhow::Result<()> {
        let range = find_top_level_string_value(&self.content, "image")
            .context("No top-level string \"image\" field to update")?;
        self.content
            .replace_range(range, &serde_json::to_string(new_image)?);
        self.image = Some(new_image.to_string());
        Ok(())
    }
}

/// Returns the byte range (including quotes) of the string value for `key`
/// in the root object of `json`, ignoring keys in nested objects.
fn find_top_level_string_value(json: &str, key: &str) -> Option<Range<usize>> {
    let bytes = json.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth = depth.saturating_sub(1),
            b'"' => {
                let end = string_end(bytes, i)?;
                let mut j = skip_whitespace(bytes, end);
                if depth == 1 && bytes.get(j) == Some(&b':') {
                    let name: String = serde_json::from_str(&json[i..end]).ok()?;
                    if name == key {
                        j = skip_whitespace(bytes, j + 1);
                        return if bytes.get(j) == Some(&b'"') {
                            Some(j..string_end(bytes, j)?)
                        } else {
                            None
                        };
                    }
                }
                i = end;
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    None
}

/// Index just past the closing quote of the string starting at `start`.
fn string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

fn skip_whitespace(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    const LCP: &str = r#"{
	"kind": "Deployment",
	"id": "liferay",
	"env": {
		"image": "not-this-one",
		"LIFERAY_NOTE": "a \"quoted\" value"
	},
	"image":   "liferaycloud/liferay-dxp:7.4-5.8.4",
	"memory": 8192
}
"#;

    #[test]
    fn reads_id_and_image() {
        let lcp = LcpJson::parse(LCP.to_string()).unwrap();
        assert_eq!(lcp.id.as_deref(), Some("liferay"));
        assert_eq!(
            lcp.image.as_deref(),
            Some("liferaycloud/liferay-dxp:7.4-5.8.4")
        );
    }

    #[test]
    fn update_only_changes_the_top_level_image() {
        let mut lcp = LcpJson::parse(LCP.to_string()).unwrap();
        lcp.update_image("liferaycloud/liferay-dxp:7.4-5.13.1")
            .unwrap();

        assert_eq!(
            lcp.content,
            LCP.replace(
                "liferaycloud/liferay-dxp:7.4-5.8.4",
                "liferaycloud/liferay-dxp:7.4-5.13.1"
            )
        );
    }

    #[test]
    fn update_fails_without_top_level_image() {
        let mut lcp = LcpJson::parse(r#"{"id": "x", "env": {"image": "y"}}"#.to_string()).unwrap();
        assert!(lcp.update_image("z").is_err());
    }
}
