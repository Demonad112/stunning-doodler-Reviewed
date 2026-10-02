# Releasing DeepServer

Releases are built by GitHub Actions (`.github/workflows/release.yml`) from a tag. Nothing has to be built locally.

## 1. Set the version

The version must be the same in three places. A unit test (`packagingConfig.test.ts`) fails if they differ:

- `package.json` → `"version"`
- `src-tauri/tauri.conf.json` → `"version"` (this one names the installers)
- `src-tauri/Cargo.toml` → `[workspace.package] version` (then run `cargo check --manifest-path src-tauri/Cargo.toml` to update `Cargo.lock`)

Merge the version bump into `main` through a pull request as usual.

## 2. Tag

```bash
git checkout main && git pull
git tag v1.0.0-rc1        # or v1.0.0 for the final release
git push origin v1.0.0-rc1
```

The tag without its `v` and its `-suffix` must match the version above, or the workflow stops at **Verify tag and version**.

To re-run a release for an existing tag, use **Actions → Release → Run workflow** and enter the tag. A run without an existing tag is rejected.

## 3. What the workflow does

1. **Verify:** the tag exists, looks like `vX.Y.Z[-suffix]`, and matches the version.
2. **Build** (`build-windows.yml`, the same workflow CI runs on pull requests):
   - builds the Disk Usage engine and runs its fork tests and the real-engine `diskusage-core` test
   - `tauri build` gives the standard installer; `tauri bundle` re-packages the same binaries with WebView2 bundled, giving the offline/server installer
   - the portable zip and `SHA256SUMS.txt`
   - checks that the offline installer really contains the WebView2 runtime
   - a silent install and uninstall of **both** installers on the runner
3. **Publish:** creates the GitHub Release and uploads the four assets.
   - `-alpha`, `-beta` and `-rc` tags are published right away as a **pre-release**.
   - Final tags (`v1.0.0`) are created as a **draft**. Open the release on GitHub, check it, and press **Publish**.
   - Re-running for the same tag replaces the assets.

## 4. After publishing

Follow `docs/install.md` on a clean machine:

- silent-install the offline installer on a Windows Server VM or Windows Sandbox (`/S`)
- open the app, run a Disk Usage scan and a Transfer Monitor copy, and use the Explorer menus
- silent-uninstall, then check that `C:\Program Files\DeepServer` and the Explorer menus are gone

## First release (v1.0.0-rc1): what to watch

The release workflow had never run before 1.0.0-rc1. The tag check and the release notes were dry-run locally, and actionlint is clean. Anything that only happens on GitHub (the hand-off between jobs, the artifact download, `gh release`) first runs for real here.

1. Push the tag from a clone with push rights, on the `main` commit you want to ship. Use the steps in section 2. Don't create the release in the GitHub UI first: the workflow would then only upload assets, and it would keep the UI's draft and pre-release settings.
2. **Verify tag and version** should log `Releasing v1.0.0-rc1 (version 1.0.0)`.
3. **Build** takes about 25 minutes. Without signing secrets it logs a notice that signing is off. Both `Test-Installer.ps1` steps must pass.
4. **Publish GitHub Release** lists `out/` and passes `sha256sum -c`. It then creates a **pre-release** with four assets and a notes table that names the same files.
5. Download the four files and check them: `sha256sum -c SHA256SUMS.txt` (or `Get-FileHash` on Windows).
6. If only Publish failed, fix it on `main`. Then re-run with **Actions → Release → Run workflow**, tag `v1.0.0-rc1`. A re-run replaces the assets, and the tag doesn't need to move. A failed Verify or Build that needs a code change needs a new tag (`v1.0.0-rc2`).

## Code signing (optional)

Without signing, everything still builds; SmartScreen warns on first run and the workflow logs a notice. To sign, add **one** of these sets of repository secrets (**Settings → Secrets and variables → Actions**). The names are the same ones altWinDirStat used.

| Method                | Secrets                                                                                                               |
| --------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Azure Trusted Signing | `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `SIGNING_ENDPOINT`, `SIGNING_ACCOUNT`, `SIGNING_PROFILE` |
| Code-signing `.pfx`   | `SIGNING_PFX_BASE64` (the `.pfx` file, base64-encoded) and `SIGNING_PFX_PASSWORD`                                     |

Azure wins when both are set. Only release builds sign; pull-request builds stay unsigned. `scripts/windows/sign.ps1` signs the engine before it's bundled, and Tauri calls it (through `bundle.windows.signCommand`) for `DeepServer.exe` and both installers.

To encode a `.pfx`:

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes('C:\path\cert.pfx')) | Set-Clipboard
```
