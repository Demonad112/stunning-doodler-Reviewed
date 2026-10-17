#!/usr/bin/env bash
# SessionStart: print repo state in a few lines (stdout becomes session context).
# Runs in the session's own directory. Don't cd to CLAUDE_PROJECT_DIR: in a worktree that is the main checkout.
git fetch -q 2>/dev/null
echo "HEAD: $(git rev-parse --short HEAD) on $(git branch --show-current); origin/main: $(git rev-parse --short origin/main) ($(git rev-list --left-right --count HEAD...origin/main | awk '{print $1" ahead, "$2" behind"}'))"
echo "Open PRs: $(gh pr list --json number,title --jq 'map("#\(.number) \(.title)") | join("; ")' 2>/dev/null)"
echo "Latest release: $(gh release list --limit 1 2>/dev/null | cut -f1,2,3 | tr '\t' ' ')"
echo "Read docs/HANDOFF.md first."
exit 0
