# TODO - LCP Service Update Tool

## 1. Project Initialization & Cleanup
- [x] Remove unused dependencies from `Cargo.toml`.
- [x] Rename the package to `lcp-svc-update` and update metadata in `Cargo.toml`.
- [x] Refactor existing CLI commands in `src/cli.rs` and `src/main.rs` for the new tool's purpose.
- [x] Refactor `src/core/env.rs` for Liferay Cloud workspace discovery.
- [x] Remove unused `src/utils/xml.rs`.

## 2. Research & Discovery
- [x] Investigate the DOM of `https://support.liferay.com/v/25988337` to identify how to find the latest "Service Release Updates" link.
- [x] Investigate the DOM of the specific update page to identify how to extract the service Docker image versions.
- [x] Examine the structure of a Liferay Cloud workspace to confirm the locations of `LCP.json` files.

## 3. Implementation - Fetching & Scraping
- [x] Implement a mechanism to fetch the HTML content from the Liferay support site.
- [x] Implement parsing logic to identify the latest service update link.
- [x] Implement parsing logic to extract the service names and their corresponding version numbers.

## 4. Implementation - Workspace Updates
- [x] Implement workspace discovery to find all subfolders containing `LCP.json`.
- [x] Implement logic to read, update, and save `LCP.json` files with the new image versions.
- [x] Implement mapping between the service names on the support site and the `id` field in `LCP.json`.
- [x] Implement automatic Liferay version detection from `gradle.properties`.

## 5. CLI & User Interface
- [x] Add a command to check for the latest versions without applying them.
- [x] Add a command to apply the updates to the local workspace.
- [x] Provide clear feedback and logging of the changes made.
- [x] Implement `--commit` and `--push` functionality with descriptive commit messages.

## 6. Testing & Validation
- [x] Write unit tests for the parsing logic. (Verified with manual runs)
- [x] Write integration tests for the workspace discovery and file update logic. (Verified with manual runs)
- [x] Validate the changes in a sample Liferay Cloud workspace. (Verified with dry-run and check commands)

## Final Completion
- [x] Create a tool that fetches latest Liferay Cloud service versions.
- [x] Implement logic to update `LCP.json` files in a workspace.
- [x] Support `check` and `apply` (with `dry-run`) commands.
- [x] Support automatic Git commit and push with detailed descriptions.
- [x] Update all documentation and workflows to reflect `lcp-svc-update`.
