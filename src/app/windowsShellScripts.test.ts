import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const scriptRoot = resolve(process.cwd(), 'scripts/windows')
/** `${VerbKey}` as it appears in the scripts. */
const verbKey = ['$', '{VerbKey}'].join('')

describe('windows shell extension scripts', () => {
  it('registers current-user file and directory context menu entries', () => {
    const script = readFileSync(resolve(scriptRoot, 'register-shell-extension.ps1'), 'utf8')

    expect(script).toContain('[string]$AppPath')
    expect(script).toContain('HKCU:\\Software\\Classes\\*\\shell\\$VerbKey')
    expect(script).toContain('HKCU:\\Software\\Classes\\Directory\\shell\\$VerbKey')
    expect(script).toContain('SelectLeft')
    expect(script).toContain('Compare with $ProductName')
    expect(script).toContain('Select Left File for Compare')
    expect(script).toContain('Select Left Folder for Compare')
    expect(script).toContain('--shell-compare')
    expect(script).toContain('--select-left')
    expect(script).toContain('%1')
    expect(script).toContain(`HKCU:\\Software\\Classes\\Directory\\shell\\${verbKey}CopyVerify`)
    expect(script).toContain(`HKCU:\\Software\\Classes\\Drive\\shell\\${verbKey}CopyVerify`)
    expect(script).toContain('--copy-verify')
  })

  it('unregisters current-user file and directory context menu entries', () => {
    const script = readFileSync(resolve(scriptRoot, 'unregister-shell-extension.ps1'), 'utf8')

    expect(script).toContain('Remove-Item')
    expect(script).toContain('HKCU:\\Software\\Classes\\*\\shell\\$VerbKey')
    expect(script).toContain('HKCU:\\Software\\Classes\\Directory\\shell\\$VerbKey')
    expect(script).toContain('SelectLeft')
    expect(script).toContain(`HKCU:\\Software\\Classes\\Drive\\shell\\${verbKey}CopyVerify`)
  })

  it('packages portable Windows release artifacts', () => {
    const script = readFileSync(resolve(scriptRoot, 'package-portable.ps1'), 'utf8')

    expect(script).toContain('corepack pnpm tauri:build')
    expect(script).toContain('DeepServer.exe')
    expect(script).toContain('deepserver-cli.exe')
    expect(script).toContain('deepserver-diskusage.exe')
    expect(script).toContain('Compress-Archive')
    expect(script).toContain('DeepServer_')
    expect(script).toContain('_x64_portable.zip')
  })
})
