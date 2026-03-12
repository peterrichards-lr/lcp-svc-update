# Liferay Services Updater (lcp-svc-update)

[![Release](https://github.com/peterrichards-lr/lcp-svc-update/actions/workflows/release.yml/badge.svg)](https://github.com/peterrichards-lr/lcp-svc-update/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A specialized CLI tool for Liferay Cloud developers to automatically update service Docker image references in their workspace.

## Motivation

Managing Liferay Cloud workspaces involves keeping multiple services (`liferay`, `database`, `search`, etc.) updated with the latest security patches and features. Traditionally, this requires checking the official Liferay Support changelog and manually updating multiple `LCP.json` files. 

**Liferay Services Updater** automates this process by:
1. Scraping the latest release data directly from Liferay Support.
2. Intelligently matching versions based on your specific Liferay Workspace version.
3. Performing surgical updates to your JSON files while preserving formatting.

## Features

- **Automated Discovery:** Identifies the latest Liferay Cloud service versions by scraping the official [Liferay Support Changelog](https://support.liferay.com/v/25988337).
- **Workspace Aware:** Detects your Liferay Workspace version (7.4, 7.3, etc.) from `gradle.properties` to ensure compatible image matching.
- **Surgical Updates:** Safely modifies `LCP.json` files for `liferay`, `database`, `search`, `webserver`, `backup`, and `ci` services.
- **Git Integration:** Automatically stages changes, creates descriptive commits, and pushes to your remote repository.
- **Dry Run Support:** Preview exactly what would be changed before any files are modified.

## Quick Demo

```text
$ lcp-svc-update check
Fetching latest service versions from Liferay Support...
Detected Liferay Workspace version: 7.4

Checking workspace at: "/path/to/my-lxc-workspace"
  [UPDATE AVAILABLE] Service: database
    Current: liferaycloud/database:5.6.3
    Latest:  liferaycloud/database:5.12.0
  [UPDATE AVAILABLE] Service: liferay
    Current: liferaycloud/liferay-dxp:7.4-5.8.4
    Latest:  liferaycloud/liferay-dxp:7.4-5.9.0
  [OK] Service: search is up to date.
```

## Installation

### macOS (Recommended)
Install via Homebrew to avoid "Unidentified Developer" warnings:
```bash
brew tap peterrichards-lr/tap
brew install lcp-svc-update
```

### From Source
Ensure you have Rust and Cargo installed, then:
```bash
cargo install --path .
```

## Usage

### Check for Updates
Scan your workspace and compare current image versions with the latest available releases:
```bash
lcp-svc-update check
```

### Apply Updates
Update all `LCP.json` files to the latest versions:
```bash
lcp-svc-update apply
```

### Update and Commit
Update, commit with a detailed summary of version changes, and push to remote:
```bash
lcp-svc-update apply --commit
```

## How it Works

1. **Scraping:** Navigates to the support portal to find the most recent Service Release Updates.
2. **Detection:** Reads `liferay/gradle.properties` to identify the target product version (e.g., `7.4`).
3. **Matching:** Maps support site table entries to workspace service IDs (`id` in `LCP.json`).
4. **Serialization:** Uses `serde_json` to preserve the structure of your `LCP.json` while only updating the `image` field.

## License
MIT - See [LICENSE](LICENSE) for details.
