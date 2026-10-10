# Cloud sessions and CI notes

Moved out of the root `CLAUDE.md` to keep the always-loaded context small.

## Cloud session limits (don't retry these)

| Blocked                                          | Use instead                                                      |
| ------------------------------------------------ | ---------------------------------------------------------------- |
| `git push` of tags, `git push --delete`          | `release.yml` with `create_tag: true`; Repo maintenance workflow |
| Delete branch / edit release (no connector tool) | Repo maintenance workflow; owner via GitHub UI                   |
| `rm` with a relative glob after `cd`             | absolute paths, or `git rm` / `git clean -n` first               |
| Editing `.github/workflows/*` in auto mode       | allowed in `.claude/settings.json`; if still refused, ask once   |

## Notes

- The `.claude/settings.json` allow-list still names cloud-only `mcp__github__*` tools. Locally `gh` is authenticated (`Demonad112`), and the GitHub connector fails here (bad auth header), so use `scripts/gh-status.sh` or `gh` through Bash.
- The V2 app crate needs WebKitGTK to build on Linux. In a cloud container without it, check with `rustup target add x86_64-pc-windows-msvc` and `cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings` (also covers `#[cfg(windows)]` code).
- Code under `#[cfg(windows)]` isn't seen by Linux clippy: re-read it by hand, or run `PREPUSH_WINDOWS=1 scripts/prepush.sh` where the msvc target is installed.
