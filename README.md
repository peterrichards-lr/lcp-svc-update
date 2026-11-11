# Liferay Services Updater (lcp-svc-update)

[![Release](https://github.com/peterrichards-lr/lcp-svc-update/actions/workflows/release.yml/badge.svg)](https://github.com/peterrichards-lr/lcp-svc-update/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A specialized CLI tool for Liferay Cloud developers to automatically update service Docker image references in their workspace.

## Motivation

Managing Liferay Cloud workspaces involves keeping multiple services (`liferay`, `database`, `search`, etc.) updated with the latest security patches and features. Traditionally, this requires checking the official Liferay Cloud services changelog and manually updating multiple `LCP.json` files. 

**Liferay Services Updater** automates this process by:
1. Reading the latest release notes directly from Liferay Learn.
2. Intelligently matching versions based on your specific Liferay Workspace version.
3. Performing surgical updates to your JSON files, changing only the image value.

## Features

- **Automated Discovery:** Identifies the latest Liferay Cloud service versions from the official [Liferay Cloud Services Changelog](https://learn.liferay.com/w/updates/release-notes/cloud-infrastructure/services-changelog).
- **Configurable Source:** The changelog location can be overridden with a flag or environment variable, so a moved page doesn't require a new release.
- **Workspace Aware:** Detects your Liferay Workspace version (7.4, 7.3, etc.) from `gradle.properties` to ensure compatible image matching.
- **Surgical Updates:** Rewrites only the top-level `image` value in `LCP.json` files (key order, indentation and everything else is untouched) for `liferay`, `database`, `search`, `webserver`, `backup`, and `ci` services.
- **Git Integration:** Optionally commits the updated files with a descriptive message, and optionally pushes.
- **Dry Run Support:** Preview exactly what would be changed before any files are modified.

## Quick Demo

```text
$ lcp-svc-update check
Finding latest service release notes...
Reading service images from: https://learn.liferay.com/w/updates/release-notes/cloud-infrastructure/services-changelog/2026.q4/2026-11-10-service-release-updates
Checking workspace at: "/path/to/my-lxc-workspace"
Project type: Liferay Cloud (LXC/DXP Cloud)
Detected Liferay version: 7.4
  [UPDATE AVAILABLE] Service: database
    Current: liferaycloud/database:5.6.3
    Latest:  liferaycloud/database:5.17.3
  [UPDATE AVAILABLE] Service: liferay
    Current: liferaycloud/liferay-dxp:7.4-5.8.4
    Latest:  liferaycloud/liferay-dxp:7.4-5.13.1
  [OK] Service: search is up to date.
```

## Installation

Installing via **Homebrew** or **Scoop** builds from source, which avoids "Unidentified Developer" warnings on macOS and gives a user-level installation on Windows.

### macOS / Linux (Homebrew)
```bash
brew tap peterrichards-lr/tap
brew install lcp-svc-update
```

### Windows (Scoop)
```powershell
scoop bucket add peterrichards-lr https://github.com/peterrichards-lr/scoop-bucket
scoop install lcp-svc-update
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

### Preview Updates
```bash
lcp-svc-update apply --dry-run
```

### Update and Commit
Update and commit with a detailed summary of version changes:
```bash
lcp-svc-update apply --commit
```

Add `--push` to also push the commit to the remote. Only the updated `LCP.json` files are committed; anything else you have staged is left alone. The tool refuses to commit (and makes no changes) if any of the files it would update already have uncommitted changes.

## Configuration

By default the tool reads the [Liferay Cloud Services Changelog](https://learn.liferay.com/w/updates/release-notes/cloud-infrastructure/services-changelog), finds the newest release notes, and reads the Docker images table from it. If Liferay moves these pages, point the tool at the new location without waiting for a new release:

| Flag | Environment variable | Purpose |
| --- | --- | --- |
| `--changelog-url <URL>` | `LCP_SVC_UPDATE_CHANGELOG_URL` | Changelog index used to discover the latest release notes |
| `--release-url <URL>` | `LCP_SVC_UPDATE_RELEASE_URL` | Read images from this specific release notes page and skip discovery |

```bash
# Use a specific release
lcp-svc-update check --release-url https://learn.liferay.com/w/updates/release-notes/cloud-infrastructure/services-changelog/2026.q4/2026-11-10-service-release-updates

# Or set it for every run
export LCP_SVC_UPDATE_CHANGELOG_URL=https://example.com/new/services-changelog
```

## How it Works

1. **Discovery:** Reads the changelog index for the newest dated release notes. Because the index can lag behind new publications, it also checks the dated URL (`<year>.q<n>/<yyyy-mm-dd>-service-release-updates`) for each day since, up to today.
2. **Detection:** Reads `liferay/gradle.properties` to identify the target product version (e.g., `7.4`).
3. **Matching:** Maps release table entries to workspace service IDs (`id` in `LCP.json`). The `liferay` service is matched on its major.minor version (e.g. `7.4`), so it never crosses DXP versions.
4. **Updating:** Replaces only the text of the top-level `image` value in each `LCP.json`, so diffs contain a single changed line.

## Releasing

1. Bump `version` in `Cargo.toml`, run `cargo build` to update `Cargo.lock`, and commit.
2. Tag the commit with the matching version and push it: `git tag v1.2.3 && git push origin v1.2.3`. The Release workflow fails if the tag doesn't match `Cargo.toml`.
3. Update the Homebrew tap (and, for a first release, the Scoop bucket) from `formula.rb.example` and `scoop.json.example`. The Gemini prompt in `.gemini/prompts/update-distribution-channels.md` automates this:
   ```bash
   "Please execute the steps in .gemini/prompts/update-distribution-channels.md to update my distribution repositories"
   ```
   The Scoop bucket picks up later releases automatically.

## License
MIT - See [LICENSE](LICENSE) for details.
