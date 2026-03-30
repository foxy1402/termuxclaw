# Main Branch Delivery Flows

This document explains what runs when code is proposed to `main` and released.

Use this with:

- [`docs/ci-map.md`](../../docs/contributing/ci-map.md)
- [`docs/pr-workflow.md`](../../docs/contributing/pr-workflow.md)
- [`docs/release-process.md`](../../docs/contributing/release-process.md)

## Branching Model

TermuxClaw uses a single default branch: `main`. All contributor PRs target `main` directly.

## Active Workflows

| File | Trigger | Purpose |
| --- | --- | --- |
| `checks-on-pr.yml` | `pull_request` -> `main` | Lint + test + build + security checks on every PR |
| `ci-run.yml` | `push`/`pull_request` -> `main` | Core CI checks for mainline changes |
| `release.yml` | `push` -> `main` | Android (Termux) release on every main commit |

## Event Summary

| Event | Workflows triggered |
| --- | --- |
| PR opened or updated against `main` | `checks-on-pr.yml`, `ci-run.yml` |
| Push to `main` (including after merge) | `ci-run.yml`, `release.yml` |

## Step-By-Step

### 1) PR -> `main`

1. Contributor opens or updates a PR against `main`.
2. `checks-on-pr.yml` and `ci-run.yml` run validation.
3. All required jobs must pass before merge.
4. Merge emits a `push` event on `main` (see section 2).

### 2) Push to `main` (including after merge)

1. Commit reaches `main`.
2. `release.yml` starts and builds Android targets:
   - `aarch64-linux-android`
   - `armv7-linux-androideabi`
3. Workflow publishes GitHub release artifacts consumed by `install.sh`.

## Quick Troubleshooting

1. **Release not appearing**: confirm commit landed on `main`; check `release.yml` run status.
2. **PR checks failing**: inspect `checks-on-pr.yml` and `ci-run.yml` logs.
3. **Installer fallback build triggered**: verify release contains Android tarballs expected by `install.sh`.
