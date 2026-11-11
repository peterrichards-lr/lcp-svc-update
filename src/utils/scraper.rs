use crate::core::ServiceImages;
use anyhow::Context;
use chrono::{Datelike, Days, NaiveDate, Utc};
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};
use std::time::Duration;

pub const DEFAULT_CHANGELOG_URL: &str =
    "https://learn.liferay.com/w/updates/release-notes/cloud-infrastructure/services-changelog";

/// How far past the newest indexed release to look for unindexed ones.
const MAX_PROBE_DAYS: u64 = 31;

pub fn client() -> anyhow::Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!("lcp-svc-update/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .build()?)
}

/// Finds the most recent release notes page under the services changelog.
///
/// The changelog navigation can lag behind newly published release notes, so
/// after reading the newest indexed release this also probes the predictable
/// `<year>.q<n>/<yyyy-mm-dd>-service-release-updates` URL for each later day.
pub fn find_latest_release_url(client: &Client, changelog_url: &str) -> anyhow::Result<String> {
    let changelog = Url::parse(changelog_url.trim_end_matches('/'))
        .with_context(|| format!("Invalid changelog URL: {}", changelog_url))?;
    let html = fetch(client, changelog.as_str())?;

    let (latest_date, latest_url) = latest_release_link(&html, &changelog)
        .with_context(|| format!("Could not find any dated release notes at {}", changelog))?;

    let today = Utc::now().date_naive();
    let earliest = latest_date
        .checked_add_days(Days::new(1))
        .into_iter()
        .chain(today.checked_sub_days(Days::new(MAX_PROBE_DAYS)))
        .max()
        .unwrap_or(today);

    let mut day = today;
    while day >= earliest {
        let url = release_url_for(&changelog, day);
        if client.head(&url).send()?.status() == StatusCode::OK {
            return Ok(url);
        }
        day = match day.pred_opt() {
            Some(d) => d,
            None => break,
        };
    }

    Ok(latest_url)
}

pub fn fetch_service_images(client: &Client, url: &str) -> anyhow::Result<ServiceImages> {
    let images = parse_service_images(&fetch(client, url)?);
    if images.is_empty() {
        anyhow::bail!("No service images found at URL: {}", url)
    }
    Ok(images)
}

fn fetch(client: &Client, url: &str) -> anyhow::Result<String> {
    let response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to fetch {}", url))?
        .error_for_status()?;
    Ok(response.text()?)
}

fn latest_release_link(html: &str, changelog: &Url) -> Option<(NaiveDate, String)> {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a[href]").unwrap();
    let prefix = format!("{}/", changelog.path());

    document
        .select(&link_selector)
        .filter_map(|link| changelog.join(link.value().attr("href")?).ok())
        .filter(|url| url.path().starts_with(&prefix))
        .filter_map(|url| {
            let slug = url.path_segments()?.next_back()?;
            let date = NaiveDate::parse_from_str(slug.get(..10)?, "%Y-%m-%d").ok()?;
            slug.contains("update").then(|| (date, url.to_string()))
        })
        .max_by_key(|(date, _)| *date)
}

fn release_url_for(changelog: &Url, date: NaiveDate) -> String {
    format!(
        "{}/{}.q{}/{}-service-release-updates",
        changelog,
        date.year(),
        date.month0() / 3 + 1,
        date.format("%Y-%m-%d")
    )
}

/// Reads every table with a "Docker Images" column. Each image cell holds one
/// or more `- repo:tag` entries, separated by `<br>` or list items.
fn parse_service_images(html: &str) -> ServiceImages {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("table").unwrap();
    let header_selector = Selector::parse("thead th").unwrap();
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let mut images = ServiceImages::default();

    for table in document.select(&table_selector) {
        let headers: Vec<String> = table
            .select(&header_selector)
            .map(|th| text(&th).to_lowercase())
            .collect();
        let Some(image_column) = headers.iter().position(|h| h.contains("docker image")) else {
            continue;
        };

        for row in table.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            let (Some(name), Some(image_cell)) = (cells.first(), cells.get(image_column)) else {
                continue;
            };
            let service_name = text(name);

            for image in image_cell
                .text()
                .flat_map(str::split_whitespace)
                .map(|token| token.trim_start_matches('-'))
                .filter(|token| token.contains('/') && token.contains(':'))
            {
                images.add(&service_name, image);
            }
        }
    }

    images
}

fn text(element: &ElementRef) -> String {
    element.text().collect::<String>().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE: &str = r#"
<table><thead><tr><th><strong>Service Name</strong></th><th><strong>Previous Release</strong></th>
<th><strong>Current Release</strong></th><th><strong>Docker Images</strong></th></tr></thead>
<tbody>
<tr><td><strong>Backup</strong></td><td>5.41.0</td><td>5.42.0</td><td>- liferaycloud/backup:5.42.0</td></tr>
<tr><td>CI</td><td>6.8.1</td><td>6.9.0</td><td>- liferaycloud/jenkins:2.580.2-lts-jdk21-6.9.0</td></tr>
<tr><td>Liferay</td><td>5.13.0</td><td>5.13.1</td><td>- liferaycloud/liferay-dxp:7.4-5.13.1<br>- liferaycloud/liferay-dxp:7.3-5.13.1</td></tr>
</tbody></table>
<table><thead><tr><th>Service Name</th><th>Service Version</th><th>Ticket No.</th><th>Description</th></tr></thead>
<tbody><tr><td>Backup</td><td>5.42.0</td><td>LCD-1</td><td>Fixed a/b:c handling</td></tr></tbody></table>
"#;

    #[test]
    fn parses_images_from_release_table() {
        let images = parse_service_images(RELEASE);
        assert_eq!(
            images.find_match("backup", "liferaycloud/backup:5.41.0", None),
            Some("liferaycloud/backup:5.42.0")
        );
        assert_eq!(
            images.find_match("ci", "liferaycloud/jenkins:old", None),
            Some("liferaycloud/jenkins:2.580.2-lts-jdk21-6.9.0")
        );
        assert_eq!(
            images.find_match("liferay", "liferaycloud/liferay-dxp:7.3-5.0.0", None),
            Some("liferaycloud/liferay-dxp:7.3-5.13.1")
        );
    }

    #[test]
    fn finds_latest_dated_link_under_changelog() {
        let changelog = Url::parse(DEFAULT_CHANGELOG_URL).unwrap();
        let html = r#"
<a href="/w/updates/release-notes/cloud-infrastructure/services-changelog/2026.q4">2026.q4</a>
<a href="/w/updates/release-notes/cloud-infrastructure/services-changelog/2026.q4/2026-11-03-service-release-updates">a</a>
<a href="/w/updates/release-notes/cloud-infrastructure/services-changelog/2026.q4/2026-10-29-service-release-updates">b</a>
<a href="/w/somewhere-else/2027-01-01-service-release-updates">c</a>
"#;
        let (date, url) = latest_release_link(html, &changelog).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 11, 3).unwrap());
        assert!(url.ends_with("/2026.q4/2026-11-03-service-release-updates"));
    }

    #[test]
    fn builds_quarterly_release_url() {
        let changelog = Url::parse(DEFAULT_CHANGELOG_URL).unwrap();
        assert_eq!(
            release_url_for(&changelog, NaiveDate::from_ymd_opt(2026, 11, 10).unwrap()),
            format!(
                "{}/2026.q4/2026-11-10-service-release-updates",
                DEFAULT_CHANGELOG_URL
            )
        );
    }
}
