# Fix Plan 0 — Redesign visual/QA pass + agent-drivable E2E

Status: Revised 2026-07-16. §0 and §8 are **done** (landed while diagnosing the boot crash);
everything else is planned. Scope: fixes only — no new features.

## Core question

The QA symptoms are not independent bugs. They trace to two decisions made during the fidelity
build-out, and this plan reverses both:

1. **The redesign hand-rolled a parallel component kit** (`fidelity-switch`, `fidelity-checkbox`,
   raw `<select>`, raw `@radix-ui/*` dialogs/menus) instead of building on the shadcn/ui layer
   that already exists in `src/components/ui/*`. Every "not shadcn", "native-looking", and
   "broken geometry" symptom lives in that hand-rolled layer.
2. **The redesign invented a second token vocabulary** (`--mk-*` in `fidelity.css`) beside the
   shadcn token set already wired through Tailwind in `src/assets/style.css` (`--background`,
   `--primary`, … via `@theme inline`). Components using `--mk-*` are invisible to the shadcn
   theme pipeline, and components using shadcn utilities don't see fidelity's palette — the
   "didn't follow the theme" symptoms are exactly this split.

**Decision: the shadcn token set is canonical.** The Fidelity Modern palette becomes *values*
assigned to the shadcn variables; `--mk-*` is retired. Fidelity components stay as thin styled
wrappers over shadcn primitives; the OS mica surface is the background.

## 0. Boot crash — FIXED 2026-07-16

**Symptom:** `dev:app` exits with code 1 after ~10s; `Failed to unregister class
Chrome_WidgetWin_0. Error = 1411` in the log; no window ever appears.

**Diagnosed via the §8 CDP setup** (this was its first real use). Causal chain:
a stale `localStorage['modkeeper-use-legacy-ui'] = 'true'` made `__root.tsx` render the legacy
tree → legacy `LibraryInit` called a renamed backend command (`TypeError: fn is not a function`)
→ `get_library_workspace` never fired → the init watchdog (`lib.rs`) hit
`std::process::exit(1)` with the window still hidden → WebView2 torn down uncleanly (the 1411
line is fallout, not the cause).

**Landed fixes:**
- §10a legacy toggle removed entirely: `__root.tsx` / `settings.lazy.tsx` / `library.index.tsx`
  always render the redesign tree; settings row, `useLegacyUiAtom`, `legacy-ui-storage.ts`, and
  the i18n strings deleted. (The legacy tree was already accepted as runtime-broken; a persisted
  flag booting into it was pure hazard.)
- Watchdog no longer kills the process: on timeout it logs and **shows the window** so a
  frontend boot failure is a visible error screen, not a silent exit-1.
- Bonus found by the clean boot: backend default settings carried `language: "en"` but lingui
  catalogs are region-tagged (`en-US`) — fixed the default, added a value migration in
  `store::load_from`, and `applyLanguage` now falls back to `en-US` instead of warning forever.

## 1. Switch thumb escapes the track

**Symptom:** the white ball indicating active state renders outside the toggle's boundary.

**Cause:** `src/redesign/shared/components/fidelity-switch.tsx:40-45` — the thumb `<span>` is
`absolute` with only `top-0.5`. No `left` is set, so its horizontal position falls back to its
static-flow position (centered inside the button), and `translate-x-[1.375rem]` then pushes it
past the 44px track.

**Fix:** replace `FidelitySwitch` internals with the existing shadcn `Switch`
(`src/components/ui/switch.tsx` — thumb geometry handled by the Radix primitive). Keep the
`FidelitySwitch` prop surface (`busy`, mandatory `aria-label`) so call sites
(`mod-title-card.tsx`, `settings-screen.tsx`) don't change.

## 2. Native `<select>` dropdowns

**Symptom:** OS-native dropdown styling, visually out of place.

**Cause:** two raw `<select>` elements:
- `src/redesign/settings/language-select.tsx`
- `src/redesign/library/mod-grid-toolbar.tsx` (type filter)

**Fix:** replace both with shadcn `Select` (`src/components/ui/select.tsx`). With the token
revert (§3) it needs no per-use restyling — it already renders from `--popover`/`--border`/etc.

## 3. Token revert: retire `--mk-*`, express Fidelity as shadcn token values

**Symptom family:** components ignoring theme/accent changes; two disconnected color systems.

**Change:**
1. Rewrite `fidelity.css` to assign the Fidelity Modern palette to the **shadcn variables** in
   `:root` / `.dark` (overriding the stone defaults in `src/assets/style.css`), deleting the
   `--mk-*` block. Mapping:

   | was (`--mk-*`) | becomes (shadcn) |
   | --- | --- |
   | `--mk-surface` | `--background` |
   | `--mk-surface-container(-hover)` | `--card` (hover via `bg-card/…` opacity or `--accent`) |
   | `--mk-surface-strong` | `--popover` |
   | `--mk-outline` | `--border` / `--input` |
   | `--mk-text` / `--mk-text-muted` | `--foreground` / `--muted-foreground` |
   | `--mk-primary(-hover/-active)` | `--primary` (+ hover via `/90` opacity, shadcn idiom) |
   | `--mk-on-primary` | `--primary-foreground` |
   | `--mk-tertiary` | `--chart-2` or a single custom `--tertiary` if truly needed |
   | `--mk-danger` / `--mk-on-danger` | `--destructive` (+ `text-white` per shadcn button) |
   | `--mk-state-hover/-active` | `--accent` (+ opacity modifiers) |
   | `--mk-radius-*` | `--radius` (+ Tailwind `rounded-*` scale from `@theme inline`) |
   | `--mk-shadow-panel` | keep as one custom var or a Tailwind shadow utility |

   Keep only true extensions: `.mk-glass-*` (backdrop-filter utilities) and `.mk-scrollbar`.
   Drop `.mk-focus-ring` — shadcn's `focus-visible:ring-ring/50 ring-[3px]` idiom replaces it.
2. Migrate `src/redesign/**` classNames from `bg-[var(--mk-…)]` arbitrary values to semantic
   Tailwind utilities (`bg-background`, `text-muted-foreground`, `bg-primary
   text-primary-foreground`, `border-border`, `rounded-lg`…).
3. `applyAccent` (`settings-repository.ts`) writes `--primary` and `--ring` (and derived
   `--primary-foreground`, §4) instead of `--mk-primary*`; hover/active shades disappear — the
   shadcn `/90` opacity idiom derives them for free.
4. Guardrail: CI grep — no `--mk-` outside `fidelity.css`, no color literals in `src/redesign/**`.

**Side benefit:** the stone values in `style.css` already keep `--card`/`--popover` translucent
(e.g. `--card: oklch(… / 0.6)`), which is exactly what the mica window (§6) needs.

## 4. Unreadable text on active/primary controls

**Symptom:** active button text (including the nav's active pill) is hard to read.

**Cause:** the accent applier overrides the primary color but never its foreground: light
accents (Green `#10b981`, Orange `#f97316` in `accent-swatches.tsx`) get white text.

**Fix:** in `applyAccent`, derive `--primary-foreground` from the accent's relative luminance
(small `contrastOn(hex)` helper, no dependency). Acceptance: every swatch × (light|dark) yields
≥ 4.5:1 on filled controls — assert in a unit test over the swatch list. Also fix the hardcoded
check-mark `text-white` in `accent-swatches.tsx` with the same helper.

## 5. Navigation bar off the design system

**Symptom:** nav dock colors don't match the design system.

**Cause:** downstream of §3 (mk tokens invisible to theming) and §4 (active pill foreground).

**Fix:** re-style `bottom-navigation.tsx` with semantic utilities during the §3 migration
(`bg-popover`, active pill `bg-primary text-primary-foreground`); re-verify visually after §4 +
§6 land before touching anything else.

## 6. Library background fights the mica window

**Symptom:** the library screen paints its own background even though the window is transparent
(`tauri.conf.json`) and `apply_window_effect` applies mica — the OS surface never shows through.

**Cause:** `app-background.tsx` fills the viewport with opaque `bg-[var(--mk-surface)]` plus two
blur blobs (`desktop-shell` mounts it unconditionally). Note `style.css` already sets
`html, body { background: transparent }` — the redesign's own layer is what breaks the chain.

**Fix:** gate `AppBackground` on `!isTauri()` (it remains the stand-in surface for the browser
prototype and Storybook, which have no mica). Panels/cards keep translucent `--card`/`--popover`
fills; if legibility over busy wallpaper suffers, raise those alphas rather than reintroducing a
painted backdrop.

## 7. Remaining non-shadcn primitives (consistency pass)

| Redesign file | Uses | Replace with |
| --- | --- | --- |
| `shared/components/confirm-dialog.tsx` | `@radix-ui/react-dialog` raw | `components/ui/dialog.tsx` (or `alert-dialog.tsx` for destructive confirms) |
| `library/manage-library/manage-library-dialog.tsx` | `@radix-ui/react-dialog` raw | `components/ui/dialog.tsx` |
| `library/bulk-actions-menu.tsx` | `@radix-ui/react-dropdown-menu` raw | `components/ui/dropdown-menu.tsx` |
| `shared/components/fidelity-checkbox.tsx` | hand-rolled | `components/ui/checkbox.tsx` |
| `shared/components/fidelity-switch.tsx` | hand-rolled (§1) | `components/ui/switch.tsx` |

Rule: fidelity components may style, but the interactive primitive underneath is always the
shadcn one.

### 7a. shadcn/ui currency check (done 2026-07-16)

Local `src/components/ui/*` audited against upstream: already current-generation — Tailwind v4
`@theme inline`, oklch, `data-slot` attributes, no `forwardRef`, CVA variants, `tw-animate-css`,
CLI 3.8.x, new-york style. Outstanding:
- **Unified imports:** components import individual `@radix-ui/react-*` packages; upstream moved
  to the single `radix-ui` package (already in `package.json` deps). Migrate imports
  (`import { Slot } from 'radix-ui'` style) and drop the per-package deps.
- **Base UI became the default for NEW projects (July 2026).** Radix is explicitly not
  deprecated; **decision: stay on Radix** — no migration. Watch for CLI `add` pulling
  Base-UI-flavored components in future; pin/inspect anything newly generated.

## 8. Agent-drivable E2E — remote debugging WORKS (validated 2026-07-16)

WebView2 honors `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`; a CDP
client can then attach to `http://localhost:9222` and read console/exceptions or drive the page.
This diagnosed §0 on the first try (launch → attach → captured the legacy-tree TypeError live).

Validated flow:
```
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 bun run dev:app
# then attach: GET http://127.0.0.1:9222/json → ws://…/devtools/page/… (CDP)
```

Remaining work to productize:
1. Add `dev:e2e` script to `package.json` setting the env var (cross-env or a small bun script).
2. Check in the console-capture helper (currently scratchpad `cdp-console.ts`) under `e2e/`.
3. Minimal Playwright harness: `chromium.connectOverCDP('http://localhost:9222')`, one smoke
   spec (boot → nav dock visible → theme roundtrip), using `create_simulation_game_root` for a
   hermetic game root.
4. Watchdog note: it no longer exits (§0) — a hung frontend now means a visible window + error
   log, which is also the E2E-observable failure mode.

**Alternative (documented, not built):** `tauri-driver` + msedgedriver (WebDriver), only if
cross-platform CI E2E is needed later.

---

## Sequencing (remaining)

1. §3 token revert (mechanical but wide — do it before component swaps so new wrappers are
   written against semantic utilities once).
2. §4 accent foreground derivation (+ its unit test).
3. §1, §2, §7 shadcn primitive adoption (incl. unified `radix-ui` imports from §7a).
4. §6 background gate, then §5 nav re-verify.
5. §8 productization (scripts + Playwright smoke) can proceed in parallel any time.
