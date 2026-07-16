/*
 * Token guardrail (fix_plan_0.md §3.4): the retired `--mk-*` vocabulary must not reappear, and
 * redesign components must not carry hex color literals in class strings — colors come from the
 * shadcn tokens in fidelity.css. Data-module exceptions (the accent swatch palette, which IS color
 * data) are listed explicitly.
 */
import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'

const ROOT = join(import.meta.dir, '..')
const REDESIGN = join(ROOT, 'src', 'redesign')

const HEX_LITERAL_EXCEPTIONS = new Set([
  'src/redesign/settings/accent-palette.ts',
  'src/redesign/shared/utils/contrast.ts',
  'src/redesign/shared/utils/contrast.test.ts',
  'src/redesign/styles/fidelity.css',
  // Accent values are DATA in these modules (default/fixture settings), not styling.
  'src/redesign/data/settings-repository.ts',
  'src/redesign/data/example-data.ts',
  'src/redesign/state/settings-state.ts',
])

function* walk(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) yield* walk(path)
    else yield path
  }
}

let failed = false

for (const path of walk(REDESIGN)) {
  const rel = relative(ROOT, path).replaceAll('\\', '/')
  const content = readFileSync(path, 'utf8')

  if (rel !== 'src/redesign/styles/fidelity.css' && content.includes('--mk-')) {
    console.error(`--mk-* token in ${rel}`)
    failed = true
  }
  if (
    !HEX_LITERAL_EXCEPTIONS.has(rel) &&
    /#[0-9a-fA-F]{6}\b|#[0-9a-fA-F]{3}\b/.test(content)
  ) {
    console.error(`hex color literal in ${rel}`)
    failed = true
  }
}

if (failed) process.exit(1)
console.log('token guardrail: OK')
