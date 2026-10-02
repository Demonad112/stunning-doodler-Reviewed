---
name: release-rc
description: Publish a DeepServer release or release candidate (e.g. /release-rc v1.0.0-rc4) by dispatching the Release workflow, waiting cheaply, verifying the published assets and checksums, and reporting in a few lines.
argument-hint: <tag> [target]
---

Release `$ARGUMENTS` (first word = tag, optional second = target, default `main`).

1. Preflight: `scripts/gh-status.sh release <tag>` must fail with "Not Found"
   (tag not released yet), and `scripts/gh-status.sh runs ci.yml 1` must show
   `main` green. If either is off, stop and say why.
2. Dispatch with the GitHub connector `actions_run_trigger` (method
   `run_workflow`, workflow `release.yml`, ref `main`, inputs
   `{"tag": "<tag>", "create_tag": true, "target": "<target>"}`).
   Do not push tags: the cloud proxy rejects them.
3. Get the run id: `scripts/gh-status.sh runs release.yml 1`.
4. Spawn the `ci-watch` agent in the background with `run <id>`; do other
   work or end the turn. Do not poll in this context.
5. When it reports: on red, give its lines plus the root cause you find
   (never re-run blindly). On green, run `scripts/gh-status.sh release <tag>`.
6. Report at most 5 lines: tag, run URL, asset count, checksum result, and
   anything the owner must do (e.g. flip prerelease, delete an old rc via the
   Repo maintenance workflow). Update `docs/HANDOFF.md` if it tracks the release.
