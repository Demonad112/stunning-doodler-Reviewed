# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

DeepServer 2.0 (`V2.0/`: Compare, Record, Disk Cleanup) **replaces** V1. V1 gets no more releases. Plan: `V2.0/docs/plan.md`; install details: `V2.0/docs/install.md`. Current work: copies keep dates, attributes, permissions, owner, audit rules, streams and long names, like `robocopy /COPY:DATSO /DCOPY:DAT` (plan: `~/.claude/plans/all-tests-have-come-fizzy-raven.md`).

## State (2026-10-10)

- `main`: 5a6157c at last fetch (includes #66). Latest release **v2.0.0-rc4** (pre-release).
- **#66** (PR A, merged): `PreserveOptions`, created/accessed/modified times, attributes, folder dating, `longPathAware` manifest, short temp name for long file names, Record UI.
- **#67** (PR B, branch `v2/meta-fidelity-b`, head c93d995): CI green, mergeable, not merged. Adds `transfer-core/src/secure.rs` (privileges, backup-semantics open, ACL/owner/audit, alternate data streams), recovery script `/COPY:DATSO /DCOPY:DAT`, six "What to keep" checkboxes. Check: `scripts/gh-status.sh pr 67`.
- Open Dependabot PRs: #50, #51, #58, #59.
- Worktrees `.claude/worktrees/meta-a` and `meta-b` can go after #67 merges.

## Done this session

- PR A merged as #66 (fc97210).
- PR B built, reviewed and fixed: SDDL round-trip, owner failure no longer loses the DACL (separate steps), security set through the handle (`SetKernelObjectSecurity`, no inheritance re-propagation), FAT sources treated as "no streams".
- CI fix c93d995: backup rights are enabled only inside a record run (`RecordState::begin`) and used only after an access-denied open, so locked files and unreadable folders still report as errors on the elevated runner.

## Next

1. Owner: merge #67, then cut rc5.
2. Owner manual check on the laptop, elevated and not: custom ACL + different owner, backdated file, hidden+system file, `Set-Content -Stream x`, 300-character path. Compare with `Get-Acl`, `Get-Item | fl *Time*,Attributes`, `Get-Item -Stream *`.
3. Fix dates/permissions on files skipped as identical (Robocopy `/SECFIX` `/TIMFIX`): "Re-check" option.
4. Re-date folders that already existed or were created implicitly.
5. "Kept / Not kept" section in the Record report; metadata comparison in verify; `created_ms` / `attributes` manifest fields; `/ZB` in recovery scripts when elevated.
6. Edge cases: UNC paths with forward slashes and `\\.\` device paths only warn.
7. Older backlog: MFT fast path, "Size on disk / Size" toggle, new logo, code signing, Settings page.

## Needs the owner

- Merge #67: `gh pr merge 67 --merge` (or the GitHub UI).
- Dependabot PRs #50, #51, #58, #59.
- Stray folder `C:\Dev\projects\DeepServerV1.0\--clobber\` (untracked, delete with approval).
- Repo maintenance workflow (`dry_run: true` first) for merged branches.

## Notes

- Apply order after rename: streams, times, attributes, security. A metadata failure is a per-file warning in `ItemResult.message`; the file still counts as Copied.
- The "AI" (auto-inherited) SDDL marker is not carried over.
- New stored fields need `#[serde(default)]`.
- Windows Application Control blocks fresh test binaries locally (os error 4551): retry, or `cargo test --workspace --no-fail-fast`; CI covers the app crate and runs elevated.
- Local cargo from a worktree: `CARGO_TARGET_DIR=C:/Dev/projects/DeepServerV1.0/V2.0/src-tauri/target`.
- One unreproduced flaky `report-core` test failure was seen once; mentioned in the #67 body.
- Clippy (1.98) flags `chunks_exact(const)`: use `as_chunks`.
