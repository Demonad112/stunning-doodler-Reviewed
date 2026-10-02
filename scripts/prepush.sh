#!/usr/bin/env bash
# Fast pre-push gate: checks only what changed vs the base branch, so the
# usual late CI failures (prettier on unstaged files, rustfmt, clippy) show
# up locally in seconds instead of after a ~25-minute CI round.
#   scripts/prepush.sh [base]     (default origin/main)
# PREPUSH_WINDOWS=1 also runs `cargo clippy --target x86_64-pc-windows-msvc`
# for code behind #[cfg(windows)] (needs `rustup target add x86_64-pc-windows-msvc`).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
base="${1:-origin/main}"
git fetch -q origin "${base#origin/}" 2>/dev/null || true
mapfile -t files < <(git diff --name-only --diff-filter=ACMR "$(git merge-base "$base" HEAD)" -- . ':!native/**' ':!docs/history/**'
  git diff --name-only --diff-filter=ACMR HEAD -- . ':!native/**' ':!docs/history/**')
mapfile -t files < <(printf '%s\n' "${files[@]}" | sort -u | while read -r f; do [ -f "$f" ] && echo "$f"; done)
[ "${#files[@]}" -gt 0 ] || { echo "prepush: nothing changed vs $base"; exit 0; }

fail=0
step() { local name="$1"; shift; if out=$("$@" 2>&1); then echo "ok    $name"; else echo "FAIL  $name"; echo "$out" | tail -n 25; fail=1; fi; }

fmt=(); lint=(); rust=0
for f in "${files[@]}"; do
  case "$f" in
    *.ts | *.vue | *.js | *.mjs | *.json | *.md | *.yml | *.yaml | *.css | *.html) fmt+=("$f") ;;&
    *.ts | *.vue) lint+=("$f") ;;
    *.rs | */Cargo.toml) rust=1 ;;
  esac
done
[ "${#fmt[@]}" -gt 0 ] && step "prettier (${#fmt[@]} files)" corepack pnpm exec prettier --check --ignore-unknown "${fmt[@]}"
[ "${#lint[@]}" -gt 0 ] && step "eslint (${#lint[@]} files)" corepack pnpm exec eslint "${lint[@]}"
[ "${#lint[@]}" -gt 0 ] && step "vue-tsc" corepack pnpm exec vue-tsc --noEmit
if [ "$rust" = 1 ]; then
  m=src-tauri/Cargo.toml
  step "rustfmt" cargo fmt --all --manifest-path "$m" -- --check
  step "clippy" cargo clippy --workspace --all-targets --manifest-path "$m" -- -D warnings
  [ "${PREPUSH_WINDOWS:-0}" = 1 ] &&
    step "clippy (windows target)" cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --manifest-path "$m" -- -D warnings
fi
[ "$fail" = 0 ] && echo "prepush: all clear" || echo "prepush: fix the FAIL items before pushing"
exit "$fail"
