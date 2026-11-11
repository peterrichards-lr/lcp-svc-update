# Liferay Services Updater (lcp-svc-update)

[![Release](https://img.shields.io/github/v/release/peterrichards-lr/lcp-svc-update)](https://github.com/peterrichards-lr/lcp-svc-update/releases)
[![License](https://img.shields.io/github/license/peterrichards-lr/lcp-svc-update)](LICENSE)
[![Rust CI](https://github.com/peterrichards-lr/lcp-svc-update/actions/workflows/rust.yml/badge.svg)](https://github.com/peterrichards-lr/lcp-svc-update/actions/workflows/rust.yml)

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

### Homebrew (macOS / Linux)

```bash
brew tap peterrichards-lr/tap
brew install lcp-svc-update
```

### Scoop (Windows)

```powershell
scoop bucket add peterrichards-lr https://github.com/peterrichards-lr/scoop-bucket
scoop install lcp-svc-update
```

### WSL (Windows Subsystem for Linux)

Install [Homebrew on Linux](https://docs.brew.sh/Homebrew-on-Linux) inside your WSL distribution, then use the Homebrew commands above.

### Manual Download

Download the archive for your platform (Linux x86_64, macOS Apple Silicon or Intel, Windows x86_64) from the [Releases page](https://github.com/peterrichards-lr/lcp-svc-update/releases), extract it, and put `lcp-svc-update` on your `PATH`.

### Build from Source

Ensure you have Rust and Cargo installed, then from a clone of this repository:

```bash
cargo install --path .
```

## Usage

```bash
lcp-svc-update --help
lcp-svc-update check --help
lcp-svc-update apply --help
lcp-svc-update --version
```

Both commands run against the current directory by default; use `-w, --workspace <PATH>` to point at a different Liferay Cloud workspace.

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

Add `--push` (`-p`) to also push the commit to the remote. Only the updated `LCP.json` files are committed; anything else you have staged is left alone. The tool refuses to commit (and makes no changes) if any of the files it would update already have uncommitted changes.

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

1. **Discovery:** Reads the changelog index for the newest dated release notes. Because the index can lag behind new publications, it also checks the dated URL (`<year>.q<n>/<yyyy-mm-dd>-service-release-updates`) for each day since, up to today (looking back at most 31 days).
2. **Detection:** Reads `liferay/gradle.properties` to identify the target product version (e.g., `7.4`).
3. **Matching:** Maps release table entries to workspace service IDs (`id` in `LCP.json`). The `liferay` service is matched on its major.minor version (e.g. `7.4`), so it never crosses DXP versions.
4. **Updating:** Replaces only the text of the top-level `image` value in each `LCP.json`, so diffs contain a single changed line.

## Releasing

Activate the shared git hooks once per clone (they run `scripts/doctor.py`, `cargo fmt` and `cargo clippy` before each commit):

```bash
git config core.hooksPath .githooks
```

1. Bump `version` in `Cargo.toml` and run `cargo build` to update `Cargo.lock`.
2. Move the `[Unreleased]` entries in `CHANGELOG.md` into a new `## [x.y.z] - YYYY-MM-DD` section.
3. Commit, then create an annotated tag and push it:
   ```bash
   python3 scripts/release.py --tag vX.Y.Z
   git push origin vX.Y.Z
   ```
   The release workflow fails if the tag isn't annotated, doesn't match `Cargo.toml`, or has no `CHANGELOG.md` section. That section becomes the GitHub release notes.
4. Update the Homebrew tap from `formula.rb.example`. The Gemini prompt in `.gemini/prompts/update-distribution-channels.md` automates this:
   ```bash
   "Please execute the steps in .gemini/prompts/update-distribution-channels.md to update my distribution repositories"
   ```
   The Scoop bucket picks up later releases automatically once a tool's manifest exists.

## Disclaimer

**This tool is provided "as-is", without warranty of any kind. `apply` rewrites `LCP.json` files in your workspace, and with `--commit` / `--push` it creates git commits and pushes them to your remote.** Run `check` or `apply --dry-run` first, review the changes before deploying them to Liferay Cloud, and make sure the new service images are compatible with your environment. You use it at your own risk.

## License

MIT - See [LICENSE](LICENSE) for details.
