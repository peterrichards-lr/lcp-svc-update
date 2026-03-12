# Liferay Services Updater (lcp-svc-update)

A specialized CLI tool for Liferay Cloud developers to automatically update service Docker image references in their workspace.

## Features

- **Automated Discovery:** Identifies the latest Liferay Cloud service versions by scraping the official Liferay Support site.
- **Workspace Aware:** Detects your Liferay Workspace version (7.4, 7.3, etc.) from `gradle.properties` to ensure compatible image matching.
- **Surgical Updates:** Safely modifies `LCP.json` files for `liferay`, `database`, `search`, `webserver`, `backup`, and `ci` services.
- **Git Integration:** Automatically stages changes, creates descriptive commits, and pushes to your remote repository.
- **Dry Run Support:** Preview exactly what would be changed before any files are modified.

## Installation

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
Or specify a path:
```bash
lcp-svc-update check --workspace /path/to/workspace
```

### Apply Updates
Update all `LCP.json` files to the latest versions:
```bash
lcp-svc-update apply
```

### Dry Run
See what would be updated without making any changes:
```bash
lcp-svc-update apply --dry-run
```

### Update and Commit
Update, commit with a detailed summary of version changes, and push to remote:
```bash
lcp-svc-update apply --commit
```

## How it Works

1. **Scraping:** Navigates to the [Liferay Support Changelog](https://support.liferay.com/v/25988337) to find the most recent Service Release Updates.
2. **Detection:** Reads `liferay/gradle.properties` to identify the target product version (e.g., `7.4`).
3. **Matching:** Maps support site table entries to workspace service IDs (`id` in `LCP.json`).
4. **Serialization:** Uses `serde_json` to preserve the structure of your `LCP.json` while only updating the `image` field.

## Development

```bash
# Build locally
cargo build

# Run tests
cargo test
```

## License
MIT - See [LICENSE](LICENSE) for details.
