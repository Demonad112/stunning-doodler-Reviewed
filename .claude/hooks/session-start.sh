#!/usr/bin/env bash
# SessionStart: print repo state in a few lines (stdout becomes session context).
# The three network calls run in parallel, each capped at 6s, so a slow network can't stall startup.
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
tmp="$(mktemp -d)" || exit 0
trap 'rm -rf "$tmp"' EXIT
timeout 6 git fetch -q 2>/dev/null &
timeout 6 gh pr list --json number,title --jq 'map("#\(.number) \(.title)") | join("; ")' >"$tmp/prs" 2>/dev/null &
timeout 6 gh release list --limit 1 >"$tmp/rel" 2>/dev/null &
wait
echo "HEAD: $(git rev-parse --short HEAD) on $(git branch --show-current); origin/main: $(git rev-parse --short origin/main) ($(git rev-list --left-right --count HEAD...origin/main | awk '{print $1" ahead, "$2" behind"}'))"
echo "Open PRs: $(cat "$tmp/prs" 2>/dev/null)"
echo "Latest release: $(cut -f1,2,3 "$tmp/rel" 2>/dev/null | tr '\t' ' ')"
echo "Read docs/HANDOFF.md first."
exit 0
