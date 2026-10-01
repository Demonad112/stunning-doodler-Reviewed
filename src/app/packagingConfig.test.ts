import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

interface TauriConfig {
  productName: string
  mainBinaryName?: string
  identifier: string
  app?: {
    windows?: {
      label?: string
      title?: string
      dragDropEnabled?: boolean
    }[]
  }
  bundle: {
    targets: string[] | string
    category?: string
    publisher?: string
    homepage?: string
    icon: string[]
    license?: string
    copyright?: string
    shortDescription?: string
    longDescription?: string
    macOS?: {
      bundleName?: string
      bundleVersion?: string
      minimumSystemVersion?: string
      hardenedRuntime?: boolean
      exceptionDomain?: string
      dmg?: {
        windowSize?: {
          width: number
          height: number
        }
        appPosition?: {
          x: number
          y: number
        }
        applicationFolderPosition?: {
          x: number
          y: number
        }
      }
    }
    linux?: {
      deb?: {
        depends?: string[]
        recommends?: string[]
        provides?: string[]
        section?: string
        priority?: string
        files?: Record<string, string>
      }
      rpm?: {
        depends?: string[]
        recommends?: string[]
        provides?: string[]
        release?: string
        epoch?: number
        files?: Record<string, string>
      }
    }
    windows?: {
      webviewInstallMode?: {
        type: string
        silent?: boolean
      }
      wix?: {
        upgradeCode?: string
        language?: string
      }
      nsis?: {
        installMode?: string
        installerIcon?: string
        compression?: string
        installerHooks?: string
      }
    }
  }
}

interface PackageManifest {
  scripts: Record<string, string>
}

function readIcnsChunks(icon: Buffer): string[] {
  const chunks: string[] = []
  let offset = 8

  while (offset + 8 <= icon.length) {
    const type = icon.subarray(offset, offset + 4).toString('ascii')
    const length = icon.readUInt32BE(offset + 4)

    if (length < 8) {
      break
    }

    chunks.push(type)
    offset += length
  }

  return chunks
}

describe('packagingConfig', () => {
  it('defines DeepServer Windows installer metadata (NSIS only)', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as TauriConfig

    expect(config.productName).toBe('DeepServer')
    expect(config.mainBinaryName).toBe('DeepServer')
    expect(config.identifier).toBe('com.demonad112.deepserver')
    expect(config.app?.windows?.[0]?.title).toBe('DeepServer')
    // MSI can't run the NSIS hooks (no Explorer menu, no cleanup), so it isn't built.
    expect(config.bundle.targets).toEqual(['nsis'])
    expect(config.bundle.windows?.wix).toBeUndefined()
    expect(config.bundle.icon).toContain('icons/icon.ico')
    expect(config.bundle.publisher).toBe('Demonad112')
    expect(config.bundle.homepage).toBe('https://github.com/Demonad112/stunning-doodler-Reviewed')
    expect(config.bundle.license).toBe('Apache-2.0')
    expect(config.bundle.copyright).toContain('Open Diff Contributors')
    expect(config.bundle.windows?.webviewInstallMode).toEqual({
      type: 'downloadBootstrapper',
      silent: true,
    })
  })

  it('defines an all-users NSIS setup.exe with Explorer context-menu hooks', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as TauriConfig
    const nsis = config.bundle.windows?.nsis

    expect(nsis?.installMode).toBe('perMachine')
    expect(nsis?.installerIcon).toBe('icons/icon.ico')
    expect(nsis?.installerHooks).toBe('windows/installer-hooks.nsh')

    const hooks = readFileSync(
      resolve(process.cwd(), 'src-tauri/windows/installer-hooks.nsh'),
      'utf8',
    )

    for (const key of ['DeepServer', 'DeepServerSelectLeft']) {
      expect(hooks).toContain(`${String.raw`Software\Classes\*\shell`}\\${key}`)
      expect(hooks).toContain(`${String.raw`Software\Classes\Directory\shell`}\\${key}`)
    }
    expect(hooks).toContain('--shell-compare --select-left')
    // Copy with verification opens the Transfer Monitor on a folder or a whole drive.
    for (const root of ['Directory', 'Drive']) {
      const key = `Software\\Classes\\${root}\\shell\\DeepServerCopyVerify`

      expect(hooks).toContain(`WriteRegStr HKLM "${key}"`)
      expect(hooks).toContain(`DeleteRegKey HKLM "${key}"`)
      expect(hooks).toContain(
        `DeleteRegKey HKCU "Software\\Classes\\${root}\\shell\\\${VERB}CopyVerify"`,
      )
    }
    expect(hooks).toContain(String.raw`--copy-verify "%1"`)
    expect(hooks).toContain('NSIS_HOOK_POSTUNINSTALL')
    // The exe comes from mainBinaryName, never a hardcoded Cargo target name.
    expect(hooks).toContain(String.raw`$INSTDIR\${MAINBINARYNAME}.exe`)
    expect(hooks).not.toContain('open-diff-app.exe')
    // Uninstall also removes the per-user keys the app's own shell integration writes.
    expect(hooks).toMatch(/DeleteRegKey HKCU/)
    // Tauri already makes the desktop shortcut; a second forced one ignores the user's choice.
    expect(hooks).not.toContain('CreateShortcut')
    // Installing over OpenDiff offers to remove it, so Explorer doesn't show both menus.
    expect(hooks).toContain(String.raw`CurrentVersion\Uninstall\OpenDiff`)
  })

  it('builds an offline / server installer flavour with the WebView2 runtime bundled (E19)', () => {
    const offlineConfig = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.offline.conf.json'), 'utf8'),
    ) as TauriConfig & { bundle: { externalBin?: string[] } }

    expect(offlineConfig.bundle.externalBin).toEqual(['binaries/deepserver-diskusage'])
    expect(offlineConfig.bundle.windows?.webviewInstallMode).toEqual({
      type: 'offlineInstaller',
      silent: true,
    })
  })

  it('keeps the app, package and Cargo workspace versions in step', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as TauriConfig & { version: string }
    const manifest = JSON.parse(readFileSync(resolve(process.cwd(), 'package.json'), 'utf8')) as {
      version: string
    }
    const cargo = readFileSync(resolve(process.cwd(), 'src-tauri/Cargo.toml'), 'utf8')
    const workspaceVersion = /\[workspace\.package\]\s*\nversion = "([^"]+)"/.exec(cargo)?.[1]

    expect(manifest.version).toBe(config.version)
    expect(workspaceVersion).toBe(config.version)
  })

  it('bundles the disk-usage engine as a sidecar with its own Explorer entries', () => {
    const engineConfig = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.engine.conf.json'), 'utf8'),
    ) as { bundle: { externalBin: string[] } }
    const baseConfig = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as TauriConfig & { bundle: { externalBin?: string[] } }
    const hooks = readFileSync(
      resolve(process.cwd(), 'src-tauri/windows/installer-hooks.nsh'),
      'utf8',
    )

    // Only release builds add the engine, so `tauri dev`, clippy and tests don't need it built.
    expect(engineConfig.bundle.externalBin).toEqual(['binaries/deepserver-diskusage'])
    expect(baseConfig.bundle.externalBin).toBeUndefined()

    // Same key name the engine's own Options page writes, so the entries overlay, not duplicate.
    for (const root of ['Directory', 'Drive']) {
      expect(hooks).toContain(`Software\\Classes\\${root}\\shell\\DeepServer Disk Usage`)
    }
    expect(hooks).toContain(String.raw`"$INSTDIR\deepserver-diskusage.exe" "%1"`)
    expect(hooks).toContain('DS_DELETE_DISKUSAGE_MENU HKLM')
    expect(hooks).toContain('DS_DELETE_DISKUSAGE_MENU HKCU')
  })
  it('enables native desktop drag-drop and grants core event capability', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as TauriConfig
    const capabilityPath = resolve(process.cwd(), 'src-tauri/capabilities/default.json')
    const capability = JSON.parse(readFileSync(capabilityPath, 'utf8')) as {
      identifier: string
      windows: string[]
      permissions: string[]
    }

    expect(config.app?.windows?.[0]?.label).toBe('main')
    expect(config.app?.windows?.[0]?.dragDropEnabled).toBe(true)
    expect(existsSync(capabilityPath)).toBe(true)
    expect(capability.identifier).toBe('default')
    expect(capability.windows).toContain('main')
    expect(capability.permissions).toContain('core:default')
  })

  it('defines macOS app and DMG bundle metadata for macOS builders', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.macos.conf.json'), 'utf8'),
    ) as Pick<TauriConfig, 'bundle'>
    const baseConfig = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as TauriConfig & { version: string }

    expect(config.bundle.targets).toEqual(['app', 'dmg'])
    expect(config.bundle.category).toBe('DeveloperTool')
    expect(config.bundle.icon).toContain('icons/icon.icns')
    expect(config.bundle.macOS?.bundleName).toBe('DeepServer')
    expect(config.bundle.macOS?.bundleVersion).toBe(baseConfig.version)
    expect(config.bundle.macOS?.minimumSystemVersion).toBe('11.0')
    expect(config.bundle.macOS?.hardenedRuntime).toBe(true)
    expect(config.bundle.macOS?.exceptionDomain).toBe('github.com')
    expect(config.bundle.macOS?.dmg?.windowSize).toEqual({ width: 660, height: 400 })
    expect(config.bundle.macOS?.dmg?.appPosition).toEqual({ x: 180, y: 170 })
    expect(config.bundle.macOS?.dmg?.applicationFolderPosition).toEqual({ x: 480, y: 170 })
    const iconPath = resolve(process.cwd(), 'src-tauri/icons/icon.icns')

    expect(existsSync(iconPath)).toBe(true)

    const icon = readFileSync(iconPath)
    const iconLength = icon.readUInt32BE(4)
    const iconChunks = readIcnsChunks(icon)

    expect(icon.subarray(0, 4).toString('ascii')).toBe('icns')
    expect(iconLength).toBe(icon.length)
    expect(iconChunks).toEqual(expect.arrayContaining(['ic10', 'ic09', 'ic08']))
  })

  it('exposes a macOS bundle script for macOS release runners', () => {
    const manifest = JSON.parse(
      readFileSync(resolve(process.cwd(), 'package.json'), 'utf8'),
    ) as PackageManifest
    const script = readFileSync(resolve(process.cwd(), 'scripts/macos/package-macos.sh'), 'utf8')

    expect(manifest.scripts['tauri:build:macos']).toBe('bash scripts/macos/package-macos.sh')
    expect(script).toContain('set -euo pipefail')
    expect(script).toContain('corepack pnpm tauri build --bundles app,dmg')
    expect(script).toContain('TAURI_SIGNING_IDENTITY')
  })

  it('defines Linux deb metadata for Debian compatible builders', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.linux.conf.json'), 'utf8'),
    ) as Pick<TauriConfig, 'bundle'>

    expect(config.bundle.targets).toContain('deb')
    expect(config.bundle.category).toBe('DeveloperTool')
    expect(config.bundle.icon).toContain('icons/icon.png')
    expect(config.bundle.shortDescription).toBe('Professional file and folder comparison tool')
    expect(config.bundle.longDescription).toContain('text, folder, table, image, binary')
    expect(config.bundle.linux?.deb?.section).toBe('devel')
    expect(config.bundle.linux?.deb?.priority).toBe('optional')
    expect(config.bundle.linux?.deb?.depends).toEqual([
      'libwebkit2gtk-4.1-0',
      'libgtk-3-0',
      'libayatana-appindicator3-1',
      'librsvg2-2',
    ])
    expect(config.bundle.linux?.deb?.recommends).toEqual(['xdg-utils'])
    expect(config.bundle.linux?.deb?.provides).toEqual(['open-diff'])
    expect(config.bundle.linux?.deb?.files).toEqual({
      '/usr/share/doc/open-diff/README.md': '../README.md',
      '/usr/share/doc/open-diff/LICENSE': '../LICENSE',
    })

    const iconPath = resolve(process.cwd(), 'src-tauri/icons/icon.png')

    expect(existsSync(iconPath)).toBe(true)
    expect(readFileSync(iconPath).subarray(0, 8)).toEqual(
      Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    )
  })

  it('exposes a Linux deb bundle script for Linux release runners', () => {
    const manifest = JSON.parse(
      readFileSync(resolve(process.cwd(), 'package.json'), 'utf8'),
    ) as PackageManifest
    const script = readFileSync(resolve(process.cwd(), 'scripts/linux/package-deb.sh'), 'utf8')

    expect(manifest.scripts['tauri:build:linux:deb']).toBe('bash scripts/linux/package-deb.sh')
    expect(script).toContain('set -euo pipefail')
    expect(script).toContain('corepack pnpm tauri build --bundles deb')
  })

  it('defines Linux rpm metadata for RPM compatible builders', () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), 'src-tauri/tauri.linux.conf.json'), 'utf8'),
    ) as Pick<TauriConfig, 'bundle'>

    expect(config.bundle.targets).toContain('rpm')
    expect(config.bundle.linux?.rpm?.release).toBe('1')
    expect(config.bundle.linux?.rpm?.epoch).toBe(0)
    expect(config.bundle.linux?.rpm?.depends).toEqual(['webkit2gtk4.1', 'gtk3', 'librsvg2'])
    expect(config.bundle.linux?.rpm?.recommends).toEqual(['xdg-utils'])
    expect(config.bundle.linux?.rpm?.provides).toEqual(['open-diff'])
    expect(config.bundle.linux?.rpm?.files).toEqual({
      '/usr/share/doc/open-diff/README.md': '../README.md',
      '/usr/share/doc/open-diff/LICENSE': '../LICENSE',
    })
  })

  it('exposes a Linux rpm bundle script for Linux release runners', () => {
    const manifest = JSON.parse(
      readFileSync(resolve(process.cwd(), 'package.json'), 'utf8'),
    ) as PackageManifest
    const script = readFileSync(resolve(process.cwd(), 'scripts/linux/package-rpm.sh'), 'utf8')

    expect(manifest.scripts['tauri:build:linux:rpm']).toBe('bash scripts/linux/package-rpm.sh')
    expect(script).toContain('set -euo pipefail')
    expect(script).toContain('corepack pnpm tauri build --bundles rpm')
  })
})
