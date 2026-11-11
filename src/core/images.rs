use std::collections::HashMap;

/// Docker images published in a services release, keyed by LCP.json service id.
#[derive(Debug, Default)]
pub struct ServiceImages {
    images: HashMap<String, Vec<String>>,
}

impl ServiceImages {
    pub fn add(&mut self, service_name: &str, image: &str) {
        self.images
            .entry(service_id_for(service_name))
            .or_default()
            .push(image.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Picks the published image that should replace `current_image`.
    ///
    /// Liferay publishes one image per DXP major.minor (e.g. `liferay-dxp:7.4-5.13.1`),
    /// so it is matched on the workspace's detected version, falling back to the
    /// version prefix of the current tag. Other services prefer an image from the
    /// same repository as the current one.
    pub fn find_match(
        &self,
        service_id: &str,
        current_image: &str,
        detected_version: Option<&str>,
    ) -> Option<&str> {
        let candidates = self.images.get(service_id)?;

        if service_id == "liferay" {
            let current_version = tag(current_image).and_then(|t| t.split('-').next());
            return [detected_version, current_version]
                .into_iter()
                .flatten()
                .find_map(|v| {
                    let prefix = format!("{}-", v);
                    candidates
                        .iter()
                        .find(|c| tag(c).is_some_and(|t| t.starts_with(&prefix)))
                })
                .map(String::as_str);
        }

        let current_repo = repository(current_image);
        candidates
            .iter()
            .find(|c| repository(c) == current_repo)
            .or(candidates.first())
            .map(String::as_str)
    }
}

fn service_id_for(service_name: &str) -> String {
    let name = service_name.trim().to_lowercase();
    match name.as_str() {
        "elasticsearch" => "search".to_string(),
        "nginx" => "webserver".to_string(),
        "jenkins" => "ci".to_string(),
        _ => name,
    }
}

fn repository(image: &str) -> &str {
    image.rsplit_once(':').map_or(image, |(repo, _)| repo)
}

fn tag(image: &str) -> Option<&str> {
    image.rsplit_once(':').map(|(_, tag)| tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release() -> ServiceImages {
        let mut images = ServiceImages::default();
        images.add("Backup", "liferaycloud/backup:5.42.0");
        images.add("CI", "liferaycloud/jenkins:2.580.2-lts-jdk21-6.9.0");
        images.add("Liferay", "liferaycloud/liferay-dxp:7.4-5.13.1");
        images.add("Liferay", "liferaycloud/liferay-dxp:7.3-5.13.1");
        images.add("Search", "liferaycloud/elasticsearch:8.19.23-5.10.0");
        images
    }

    #[test]
    fn matches_liferay_on_detected_version() {
        let images = release();
        assert_eq!(
            images.find_match("liferay", "liferaycloud/liferay-dxp:7.4-5.8.4", Some("7.3")),
            Some("liferaycloud/liferay-dxp:7.3-5.13.1")
        );
    }

    #[test]
    fn matches_liferay_on_current_tag_without_detected_version() {
        let images = release();
        assert_eq!(
            images.find_match("liferay", "liferaycloud/liferay-dxp:7.3-5.8.4", None),
            Some("liferaycloud/liferay-dxp:7.3-5.13.1")
        );
    }

    #[test]
    fn does_not_cross_liferay_versions() {
        let images = release();
        assert_eq!(
            images.find_match("liferay", "liferaycloud/liferay-dxp:7.1-5.8.4", None),
            None
        );
    }

    #[test]
    fn matches_other_services_by_id() {
        let images = release();
        assert_eq!(
            images.find_match("ci", "liferaycloud/jenkins:2.452.1-jdk17-5.3.0", None),
            Some("liferaycloud/jenkins:2.580.2-lts-jdk21-6.9.0")
        );
        assert_eq!(
            images.find_match("search", "liferaycloud/elasticsearch:7.17.0-5.1.0", None),
            Some("liferaycloud/elasticsearch:8.19.23-5.10.0")
        );
        assert_eq!(images.find_match("database", "x/y:1", None), None);
    }

    #[test]
    fn service_names_map_exactly() {
        let mut images = ServiceImages::default();
        images.add("Cache Service", "liferaycloud/cache:1.0.0");
        assert_eq!(images.find_match("ci", "", None), None);
    }
}
