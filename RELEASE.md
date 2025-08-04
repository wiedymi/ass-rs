# Release Process

This document describes the automated release process for the ass-rs workspace using GitHub Actions.

## Overview

The ass-rs workspace contains two crates that are automatically published in the correct order:
1. `ass-core` - The core parsing library
2. `ass-editor` - The editor layer (depends on ass-core)

## Prerequisites

Before releasing, ensure:
- [ ] All tests pass on main branch
- [ ] CHANGELOG.md is updated with release notes
- [ ] Version numbers in Cargo.toml files are updated
- [ ] All changes are committed and pushed
- [ ] CI pipeline is green on main branch

## GitHub Setup

### Required Secrets

Set the following secret in your GitHub repository (Settings → Secrets → Actions):
- `CRATES_IO_TOKEN` - Your crates.io API token with publish permissions

## Release Process

### 1. Update Version Numbers

Update the version in the appropriate Cargo.toml files:
```toml
# crates/ass-core/Cargo.toml
version = "0.1.0"

# crates/ass-editor/Cargo.toml
version = "0.1.0"
```

### 2. Update CHANGELOG

Add your changes to CHANGELOG.md under the appropriate version heading.

### 3. Commit and Push

```bash
git add -A
git commit -m "chore: Prepare release v0.1.0"
git push origin main
```

### 4. Create and Push Tag

The tag format determines what gets released:

```bash
# Release both crates (workspace release)
git tag -a v0.1.0 -m "Release version 0.1.0"

# Release only ass-core
git tag -a ass-core-v0.1.1 -m "Release ass-core version 0.1.1"

# Release only ass-editor
git tag -a ass-editor-v0.1.1 -m "Release ass-editor version 0.1.1"

# Push the tag
git push origin --tags
```

### 5. Monitor Release

The GitHub Actions workflow will automatically:
1. Create a draft GitHub release
2. Run all tests and checks
3. Build release binaries for multiple platforms
4. Publish to crates.io in the correct order:
   - For workspace releases: publishes ass-core first, waits for indexing, then publishes ass-editor
   - For individual releases: publishes only the tagged crate
5. Generate and publish documentation

Monitor the progress in the Actions tab of your repository.

## Tag Formats

- `v[0-9]+.*` - Triggers workspace release (both crates)
- `ass-core-v[0-9]+.*` - Triggers ass-core release only
- `ass-editor-v[0-9]+.*` - Triggers ass-editor release only

## Versioning Strategy

We follow [Semantic Versioning](https://semver.org/):
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality additions
- PATCH version for backwards-compatible bug fixes

### Version Guidelines

- During initial development (0.x.y), keep both crates at the same version
- After 1.0.0, versions can diverge based on individual crate changes
- ass-editor must always depend on a compatible ass-core version

## Troubleshooting

### Release Failed

1. Check the GitHub Actions logs for specific errors
2. Common issues:
   - Version already exists on crates.io
   - Missing or invalid CRATES_IO_TOKEN
   - Tests failing on release configuration
   - Documentation build errors

### Manual Recovery

If the automated release partially fails:

1. **If ass-core published but ass-editor failed:**
   - Fix the issue
   - Create a new tag for ass-editor only
   - Push the tag to trigger just the ass-editor release

2. **If GitHub release creation failed:**
   - Manually create the release from the existing tag
   - Upload any missing artifacts

## Security Notes

- The CRATES_IO_TOKEN should have minimal required permissions
- Rotate tokens periodically
- Never commit tokens to the repository
- Use GitHub's secret scanning to detect accidental commits