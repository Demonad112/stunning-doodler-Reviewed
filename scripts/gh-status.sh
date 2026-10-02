#!/usr/bin/env bash
# Compact GitHub status: a few lines instead of raw API JSON.
# Transport: `gh api` when gh is installed (laptop, cloud), else curl + jq
# (cloud sessions, where the git proxy authenticates api.github.com).
# Never prints tokens or headers.
#
#   scripts/gh-status.sh release <tag>          state, assets, SHA256SUMS vs asset digests
#   scripts/gh-status.sh run <id>|latest [wf]   run result + first error lines of failed jobs
#   scripts/gh-status.sh runs [wf] [n]          last n runs (default 5)
#   scripts/gh-status.sh pr <n>                 state, mergeability, failing checks
#   scripts/gh-status.sh branches               remote branches whose PR merged at that exact commit
#   scripts/gh-status.sh wait run <id> [secs]   block until the run finishes (default 540 s)
#   scripts/gh-status.sh wait pr <n> [secs]     block until no check on the PR head is pending
#
# Repo: $GH_REPO, else parsed from `git remote get-url origin`.
# Exit codes: 0 ok/green, 1 red/failed, 2 usage or API error, 3 still pending after wait.
set -euo pipefail

REPO="${GH_REPO:-$(git remote get-url origin 2>/dev/null | sed -E 's#^.*github\.com[:/]##; s#\.git$##')}"
[ -n "$REPO" ] || { echo "error: set GH_REPO=owner/name" >&2; exit 2; }
ERR_LINES="${GH_STATUS_ERR_LINES:-12}"

# api <path> <jq-filter>
api() {
  if command -v gh >/dev/null 2>&1 && [ "${GH_STATUS_TRANSPORT:-}" != curl ]; then
    gh api "$1" --jq "$2"
  else
    curl -fsSL -H 'Accept: application/vnd.github+json' "https://api.github.com/$1" | jq -r "$2"
  fi
}
# raw <path> [accept]: body as-is (logs, asset contents); follows redirects
raw() {
  local accept="${2:-application/vnd.github+json}"
  if command -v gh >/dev/null 2>&1 && [ "${GH_STATUS_TRANSPORT:-}" != curl ]; then
    gh api -H "Accept: $accept" "$1"
  else
    curl -fsSL -H "Accept: $accept" "https://api.github.com/$1"
  fi
}

errlines() { # first meaningful error lines of a job log (timestamps, ANSI, warnings stripped)
  sed -E 's/\r$//; s/^[0-9T:.-]+Z //; s/\x1b\[[0-9;]*m//g' |
    grep -E -A1 '##\[error\]|(^|[^a-z])error(\[E[0-9]+\])?: |panicked at| \.\.\. FAILED|result: FAILED|^ *(✗|×|FAIL) |Error: |ERR_' |
    grep -Ev '^--$|^ *$|^\[Vue warn|^warning|::group::|^Post job| \.\.\. ok$' | head -n "$ERR_LINES" | cut -c1-220
}

cmd_release() {
  local tag="${1:?usage: release <tag>}"
  api "repos/$REPO/releases/tags/$tag" '
    "release \(.tag_name)  draft=\(.draft) prerelease=\(.prerelease)  published=\(.published_at // "-")  target=\(.target_commitish)",
    (.assets[] | "  \(.name)  \((.size/1048576*10|floor)/10) MB  \(.state)")'
  local sums_id
  sums_id=$(api "repos/$REPO/releases/tags/$tag" '.assets[] | select(.name=="SHA256SUMS.txt") | .id')
  if [ -z "$sums_id" ]; then echo "checksums: no SHA256SUMS.txt asset"; return 1; fi
  local sums digests bad=0 n=0
  sums=$(raw "repos/$REPO/releases/assets/$sums_id" application/octet-stream | tr -d '\r')
  digests=$(api "repos/$REPO/releases/tags/$tag" '.assets[] | "\(.name) \(.digest // "")"')
  while read -r hash name; do
    [ -n "$hash" ] || continue
    name="${name#\*}"; name="${name##*/}"; n=$((n + 1))
    actual=$(awk -v f="$name" '$1==f {print $2}' <<<"$digests")
    if [ -z "$actual" ]; then echo "  MISSING asset $name"; bad=1
    elif [ "${actual#sha256:}" != "${hash,,}" ]; then echo "  MISMATCH $name"; bad=1
    fi
  done <<<"$sums"
  [ "$bad" = 0 ] && echo "checksums: OK ($n files match GitHub asset digests)" || { echo "checksums: FAILED"; return 1; }
}

cmd_runs() {
  local wf="${1:-}" n="${2:-5}" path
  path="repos/$REPO/actions/runs?per_page=$n"
  [ -n "$wf" ] && path="repos/$REPO/actions/workflows/$wf/runs?per_page=$n"
  api "$path" '.workflow_runs[] | "\(.id)  \(.name)  \(.head_branch)  \(.event)  \(.status)/\(.conclusion // "-")  \(.created_at)"'
}

run_id() { # resolve "latest [wf]" to an id
  if [ "$1" = latest ]; then cmd_runs "${2:-}" 1 | awk '{print $1}'; else echo "$1"; fi
}

cmd_run() {
  local id; id=$(run_id "${1:?usage: run <id>|latest [workflow]}" "${2:-}")
  local line; line=$(api "repos/$REPO/actions/runs/$id" '"\(.status) \(.conclusion // "-") \(.name) \(.head_branch) \(.head_sha[0:7]) \(.html_url)"')
  echo "run $id: $line"
  read -r status concl _ <<<"$line"
  api "repos/$REPO/actions/runs/$id/jobs?per_page=100" '
    .jobs | group_by(.conclusion // .status) | map("  \(.[0].conclusion // .[0].status): \(length)") | .[]'
  local failed; failed=$(api "repos/$REPO/actions/runs/$id/jobs?per_page=100" \
    '.jobs[] | select(.conclusion=="failure" or .conclusion=="timed_out") | "\(.id)\t\(.name)\t\([.steps[]? | select(.conclusion=="failure") | .name] | join(", "))"')
  while IFS=$'\t' read -r jid jname jstep; do
    [ -n "$jid" ] || continue
    echo "FAILED job: $jname  (step: ${jstep:-?})"
    raw "repos/$REPO/actions/jobs/$jid/logs" 2>/dev/null | errlines | sed 's/^/    /' || echo "    (log unavailable)"
  done <<<"$failed"
  [ "$status" != completed ] && return 3
  [ "$concl" = success ] || [ "$concl" = skipped ]
}

cmd_pr() {
  local n="${1:?usage: pr <number>}" sha
  api "repos/$REPO/pulls/$n" '"PR #\(.number) \(.state)\(if .merged then " (merged)" else "" end)  draft=\(.draft)  mergeable=\(.mergeable // "computing")/\(.mergeable_state)  head=\(.head.ref)@\(.head.sha[0:7])"'
  sha=$(api "repos/$REPO/pulls/$n" '.head.sha')
  api "repos/$REPO/commits/$sha/check-runs?per_page=100" '
    .check_runs as $c |
    "checks: \($c|length) total, \([$c[]|select(.status!="completed")]|length) pending, \([$c[]|select(.conclusion=="success")]|length) ok, \([$c[]|select(.conclusion=="skipped" or .conclusion=="neutral")]|length) skipped",
    ($c[] | select(.conclusion=="failure" or .conclusion=="timed_out" or .conclusion=="cancelled" or .conclusion=="action_required") | "  RED \(.name): \(.conclusion)  \(.details_url)")'
}

pr_pending() { # prints pending count / red count on the PR head
  local sha; sha=$(api "repos/$REPO/pulls/$1" '.head.sha')
  api "repos/$REPO/commits/$sha/check-runs?per_page=100" \
    '"\([.check_runs[]|select(.status!="completed")]|length) \([.check_runs[]|select(.conclusion=="failure" or .conclusion=="timed_out" or .conclusion=="cancelled")]|length)"'
}

cmd_branches() {
  local branches merged
  branches=$(api "repos/$REPO/branches?per_page=100" '.[] | "\(.name) \(.commit.sha) \(.protected)"')
  merged=$(api "repos/$REPO/pulls?state=closed&per_page=100" '.[] | select(.merged_at) | "\(.head.ref) \(.head.sha) \(.number)"')
  local any=0
  while read -r name sha prot; do
    [ "$prot" = true ] && continue
    pr=$(awk -v b="$name" -v s="$sha" '$1==b && $2==s {print $3; exit}' <<<"$merged")
    [ -n "$pr" ] && { echo "deletable: $name  (PR #$pr merged at ${sha:0:7})"; any=1; }
  done <<<"$branches"
  [ "$any" = 1 ] || echo "no merged branches left"
}

cmd_wait() {
  local kind="${1:?usage: wait run|pr <id> [secs]}" id="${2:?}" max="${3:-540}" step=30 t=0
  while :; do
    case "$kind" in
      run) st=$(api "repos/$REPO/actions/runs/$(run_id "$id")" '.status'); [ "$st" = completed ] && break ;;
      pr) read -r pend red <<<"$(pr_pending "$id")"; [ "$pend" = 0 ] && break ;;
      *) echo "usage: wait run|pr <id> [secs]" >&2; return 2 ;;
    esac
    [ "$t" -ge "$max" ] && { echo "still pending after ${t}s"; [ "$kind" = run ] && cmd_run "$id" >/dev/null; return 3; }
    sleep "$step"; t=$((t + step))
  done
  if [ "$kind" = run ]; then cmd_run "$id"; else cmd_pr "$id"; [ "${red:-0}" = 0 ]; fi
}

case "${1:-}" in
  release | run | runs | pr | branches | wait) c="$1"; shift; "cmd_$c" "$@" ;;
  *) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
