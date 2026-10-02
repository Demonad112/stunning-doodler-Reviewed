#!/usr/bin/env bash
# PreToolUse(Bash) hook: run scripts/prepush.sh before any `git push`.
# Exit 2 blocks the push and shows the failures to Claude.
input=$(cat)
grep -Eq '"command"[^}]*git[^"]* push' <<<"$input" || exit 0
grep -q 'PREPUSH_SKIP=1' <<<"$input" && exit 0
out=$("$CLAUDE_PROJECT_DIR/scripts/prepush.sh" 2>&1) && exit 0
echo "$out" >&2
exit 2
