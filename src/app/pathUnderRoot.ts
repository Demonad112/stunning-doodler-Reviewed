/**
 * True when `path` is `root` or inside it. The backend reports forward-slash paths, while roots
 * typed or picked on Windows use backslashes and may differ in case or end with a separator.
 */
export function isPathUnderRoot(path: string, root: string): boolean {
  const normalize = (value: string): string => value.replaceAll('\\', '/').toLowerCase()
  const base = normalize(root).replace(/\/+$/, '')

  return base !== '' && `${normalize(path)}/`.startsWith(`${base}/`)
}
