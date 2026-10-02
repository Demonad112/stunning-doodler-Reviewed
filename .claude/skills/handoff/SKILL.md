---
name: handoff
description: Update docs/HANDOFF.md, the committed session state file, so the next session starts by reading it instead of a pasted prompt. Use at the end of a work block, before a long wait, or when context is getting large.
---

Rewrite `docs/HANDOFF.md` in place (keep its headings, ≤ 80 lines, facts only,
no transcript). Sections:

- **Goal**: one or two lines.
- **State**: `main` sha, open PRs / branches with `scripts/gh-status.sh pr <n>`
  one-liners, latest release.
- **Done this session**: bullets with PR/commit refs.
- **Next**: numbered, smallest first; each one a session-sized task.
- **Blocked / needs the owner**: each with the exact command or UI path.
- **Notes**: decisions and gotchas not already in `CLAUDE.md`.

Drop items that are finished and older than one session. Then commit it on
the current branch (`docs: update handoff`) and push, so it survives a
container restart. If the branch is `main`-protected, put it on the working
branch and say so. Reply with one line: the file path and the top Next item.
