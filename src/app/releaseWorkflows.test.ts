import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const workflowRoot = resolve(process.cwd(), '.github/workflows')

function readWorkflow(name: string): string {
  return readFileSync(resolve(workflowRoot, name), 'utf8')
}

/** The `on:` block: from `on:` up to the next top-level key. */
function triggers(workflow: string): string {
  return /^on:\n([\s\S]*?)^\S/m.exec(workflow)?.[1] ?? ''
}

describe('release workflows', () => {
  it('releases Windows builds only, from an existing v* tag (E20)', () => {
    const release = readWorkflow('release.yml')
    const on = triggers(release)

    expect(on).toContain("- 'v*'")
    expect(on).toContain('workflow_dispatch:')
    expect(on).toMatch(/tag:\n\s+description: .+\n\s+required: true/)
    expect(on).not.toContain('pull_request')
    expect(release).toContain('git rev-parse -q --verify "refs/tags/$tag"')
    expect(release).toContain('uses: ./.github/workflows/build-windows.yml')
    expect(release).not.toMatch(/runs-on: (macos|ubuntu-22\.04)/)
    expect(release).not.toContain('tauri-action')
  })

  it('builds both installer flavours, the portable zip and checksums in one shared workflow', () => {
    const build = readWorkflow('build-windows.yml')

    expect(build).toContain('workflow_call:')
    expect(build).toContain('src-tauri/tauri.engine.conf.json')
    expect(build).toContain('src-tauri/tauri.offline.conf.json')
    expect(build).toContain('_x64-setup.exe')
    expect(build).toContain('_x64-offline-setup.exe')
    expect(build).toContain('package-portable.ps1')
    expect(build).toContain('SHA256SUMS.txt')
    expect(build).toContain('Test-ForkChanges.ps1')
    expect(build).toContain('Test-Installer.ps1')
  })

  it('runs the Windows build and engine tests from CI and drops the old installer workflow', () => {
    const ci = readWorkflow('ci.yml')

    expect(ci).toContain('uses: ./.github/workflows/build-windows.yml')
    expect(ci).toContain('engine-upstream-tests:')
    expect(existsSync(resolve(workflowRoot, 'windows-installer.yml'))).toBe(false)
  })

  it('prunes Actions storage only when run by hand (E21)', () => {
    const on = triggers(readWorkflow('prune-storage.yml'))

    expect(on).toContain('workflow_dispatch:')
    expect(on).not.toContain('pull_request')
    expect(on).not.toContain('push:')
  })

  it('signs only when secrets are configured', () => {
    const sign = readFileSync(resolve(process.cwd(), 'scripts/windows/sign.ps1'), 'utf8')

    expect(sign).toContain("$mode -eq 'none'")
    expect(sign).toContain('/tr http://timestamp')
  })
})
