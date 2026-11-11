# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_Nothing yet._

## [1.1.2] - 2026-11-11

### Changed

- Label Dependabot pull requests `no-issue-needed`, alongside the default `dependencies` and ecosystem labels, so bot PRs are marked as intentionally having no linked issue.
- Update Rust dependencies:
  - `reqwest` 0.11.27 → 0.13.5
  - `anyhow` 1.0.102 → 1.0.104
  - `clap` 4.5.60 → 4.6.7
  - `scraper` 0.18.1 → 0.29.0
  - `serde_json` 1.0.149 → 1.0.154
- Update GitHub Actions:
  - `dtolnay/rust-toolchain` (pinned commit updated)
  - `actions/download-artifact` 8.0.1 → 8.0.2
  - `actions/upload-artifact` 7.0.1 → 7.0.2

## [1.1.1] - 2026-11-11

### Added

- Add `CHANGELOG.md`, with sections backfilled for every earlier release.
- Add `.github/dependabot.yml` for weekly `cargo` and GitHub Actions updates.
- Add the shared `.githooks/pre-commit` hook (doctor check, `cargo fmt`, `cargo clippy`) and `.cargo/config.toml` aliases (`setup-hooks`, `lint`, `fmt-check`).
- Add `scripts/doctor.py` and `scripts/release.py` from the Liferay Rust tool template (v0.7.0).

### Changed

- Replace `.github/workflows/release.yml` with the template workflow: it requires an annotated `vX.Y.Z` tag that matches the `Cargo.toml` version, publishes the matching `CHANGELOG.md` section as the release notes, derives binary names from `Cargo.toml`, and adds an Intel macOS (`x86_64-apple-darwin`) build.
- Pin GitHub Actions in `.github/workflows/rust.yml` and `release.yml` to commit SHAs.
- Update `.gemini/prompts/update-distribution-channels.md` to the template version, which names the formula and manifest after the binary and verifies the tap and bucket paths.
- Restructure `README.md` to the template standard: release, license and Rust CI badges, WSL and manual download install options, an updated Releasing section and a Disclaimer.

### Removed

- Remove the stale `todo.md` checklist.

## [1.1.0] - 2026-11-11

### Added

- Add `--changelog-url` / `LCP_SVC_UPDATE_CHANGELOG_URL` and `--release-url` / `LCP_SVC_UPDATE_RELEASE_URL` to override where service images are read from.
- Add `--push` to `apply`, which requires `--commit`.
- Add `repository` and `license` to `Cargo.toml`, plus `formula.rb.example` and Configuration and Releasing sections in the README.

### Changed

- Read service images from the Liferay Learn [Cloud Services Changelog](https://learn.liferay.com/w/updates/release-notes/cloud-infrastructure/services-changelog) instead of the Liferay Support page, which now redirects away. Release notes newer than the changelog index are found by probing dated URLs for up to 31 days.
- Update `LCP.json` surgically: only the top-level `image` value changes, keeping key order and formatting.
- `apply --commit` no longer pushes, commits only the updated `LCP.json` files, and refuses before writing anything if any of them have uncommitted changes.
- Match service names exactly, and match the `liferay` image on its DXP major.minor version so it never crosses versions.
- The release workflow fails if the tag doesn't match the `Cargo.toml` version.
- Set the `Cargo.toml` version to 1.1.0. Earlier binaries reported a stale version: 0.2.0 and 1.0.0 reported 0.1.1, and 0.1.1 reported 0.1.0.
- Bump `openssl` and `rand` to patched versions.

### Fixed

- Report unreadable `LCP.json` files instead of skipping them silently.
- Stop assuming a service's folder name equals its `id`.
- Align `scoop.json.example` with the bucket's existing manifests.

### Removed

- Remove the unused `edit-xml` and `serde_yaml` dependencies.

## [1.0.0] - 2026-03-13

### Added

- Add a Rust CI workflow (`rust.yml`) running format, Clippy and test checks.
- Show the detected project type in `check` output.

### Changed

- Split the release workflow into separate build and release jobs.

### Fixed

- Fix Clippy dead-code and unused-import warnings.

## [0.2.0] - 2026-03-12

### Added

- Add git helpers and project discovery improvements from the Liferay Rust tool template v0.2.0.
- Add Homebrew installation instructions to the README.

### Changed

- Make the web scraping dependencies an optional, default-enabled `web` feature.

## [0.1.1] - 2026-03-12

### Changed

- Track `Cargo.lock` so CI builds are reproducible.

## [0.1.0] - 2026-03-12

### Added

- First release of Liferay Services Updater, built from the Liferay Rust tool template.
- `check` compares the Docker images in a workspace's `LCP.json` files with the latest versions published by Liferay.
- `apply` updates `LCP.json` files, with `--dry-run` and optional git commit.
- Detect the Liferay Workspace version from `gradle.properties`.

<!-- markdownlint-disable MD049 -->
---
*Last Updated: 2026-11-11* | *Last Reviewed: 2026-11-11*
