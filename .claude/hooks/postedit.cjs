#!/usr/bin/env node
/* eslint-disable */
// PostToolUse Edit|Write: exit 2 with a message if the edited file has control characters or mixed CRLF/LF.
// Copy of the postedit mode of the global context-guard.js so it also runs in cloud sessions.
// Any internal error is swallowed so a bug here can never stall a session.
const fs = require('fs')
const path = require('path')

const SKIP_EXT = new Set(['.png', '.jpg', '.jpeg', '.gif', '.webp', '.pdf', '.ipynb'])

function stdinJson() {
  try {
    return JSON.parse(fs.readFileSync(0, 'utf8') || '{}')
  } catch {
    return {}
  }
}

function postedit(inp) {
  const fp = (inp.tool_input || {}).file_path
  if (!fp || SKIP_EXT.has(path.extname(fp).toLowerCase())) return ''
  let buf
  try {
    if (fs.statSync(fp).size > 1024 * 1024) return ''
    buf = fs.readFileSync(fp)
  } catch {
    return ''
  }
  const s = buf.toString('utf8')
  const problems = []
  const ctl = s.match(/[\x00-\x08\x0b\x0e-\x1f]/g)
  if (ctl)
    problems.push(
      `control character(s) ${[...new Set(ctl)].map((c) => '\\x' + c.charCodeAt(0).toString(16).padStart(2, '0')).join(' ')}`,
    )
  const crlf = (s.match(/\r\n/g) || []).length
  const lf = (s.match(/\n/g) || []).length - crlf
  if (crlf && lf) problems.push(`mixed line endings (${crlf} CRLF, ${lf} LF)`)
  return problems.length
    ? `${path.basename(fp)} looks corrupted: ${problems.join('; ')}. Fix it now.`
    : ''
}

try {
  const msg = postedit(stdinJson())
  if (msg) {
    process.stderr.write(msg)
    process.exit(2)
  }
} catch {
  /* never break the session */
}
process.exit(0)
