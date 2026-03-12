use scraper::{Html, Selector};
use std::collections::HashMap;

const BASE_URL: &str = "https://support.liferay.com";
const CHANGELOG_URL: &str = "https://support.liferay.com/v/25988337";

pub fn fetch_latest_update_link() -> anyhow::Result<String> {
    let response = reqwest::blocking::get(CHANGELOG_URL)?.text()?;
    let document = Html::parse_document(&response);
    
    let selector = Selector::parse("a").unwrap();
    
    for link in document.select(&selector) {
        if let Some(href) = link.value().attr("href") {
            let text = link.text().collect::<String>();
            if text.contains("Service Release Updates") {
                if href.starts_with('/') {
                    return Ok(format!("{}{}", BASE_URL, href));
                } else {
                    return Ok(href.to_string());
                }
            }
        }
    }

    anyhow::bail!("Could not find latest Service Release Updates link")
}

pub fn fetch_service_versions(url: &str) -> anyhow::Result<HashMap<String, String>> {
    let response = reqwest::blocking::get(url)?.text()?;
    let document = Html::parse_document(&response);
    let mut versions = HashMap::new();

    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let li_selector = Selector::parse("li").unwrap();

    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            if cells.len() >= 4 {
                let service_name = cells[0].text().collect::<String>().trim().to_lowercase();
                
                let mut images = Vec::new();
                let li_elements: Vec<_> = cells[3].select(&li_selector).collect();
                if !li_elements.is_empty() {
                    for li in li_elements {
                        images.push(li.text().collect::<String>().trim().to_string());
                    }
                } else {
                    images.push(cells[3].text().collect::<String>().trim().to_string());
                }

                for image in images {
                    if !image.is_empty() && image.contains('/') {
                        add_image_to_versions(&mut versions, &service_name, &image);
                    }
                }
            }
        }
    }

    if versions.is_empty() {
        anyhow::bail!("No service versions found at URL: {}", url)
    }

    Ok(versions)
}

fn add_image_to_versions(versions: &mut HashMap<String, String>, service_name: &str, image: &str) {
    let key = match service_name {
        s if s.contains("liferay") => "liferay",
        s if s.contains("database") => "database",
        s if s.contains("search") || s.contains("elasticsearch") => "search",
        s if s.contains("webserver") || s.contains("nginx") => "webserver",
        s if s.contains("backup") => "backup",
        s if s.contains("ci") || s.contains("jenkins") => "ci",
        _ => service_name,
    };

    if key == "liferay" {
        // Store liferay versions with their major.minor as suffix (e.g., liferay:7.4)
        if let Some(version_part) = image.split(':').nth(1) {
            if let Some(v) = version_part.split('-').next() {
                let specific_key = format!("{}:{}", key, v);
                versions.insert(specific_key, image.to_string());
            }
        }
        // Also keep a default "liferay" key (prefer 7.4)
        if image.contains("7.4") || !versions.contains_key("liferay") {
            versions.insert(key.to_string(), image.to_string());
        }
    } else {
        versions.insert(key.to_string(), image.to_string());
    }
}
