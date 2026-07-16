/*
 * Accent-foreground derivation (fix_plan_0.md §4): a filled control's text color is derived from
 * the accent's relative luminance, so light accents (green, orange) get dark text instead of the
 * hardcoded white that made them unreadable. No dependency — WCAG 2.x math only.
 */

const LIGHT_FOREGROUND = '#ffffff'
// The Fidelity dark ink, not pure black, so derived dark text matches the rest of the UI.
const DARK_FOREGROUND = '#17181c'

function channelToLinear(channel: number): number {
  const c = channel / 255
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
}

export function relativeLuminance(hex: string): number {
  const value = hex.replace('#', '')
  const full =
    value.length === 3
      ? value
          .split('')
          .map((ch) => ch + ch)
          .join('')
      : value
  const r = parseInt(full.slice(0, 2), 16)
  const g = parseInt(full.slice(2, 4), 16)
  const b = parseInt(full.slice(4, 6), 16)
  return (
    0.2126 * channelToLinear(r) +
    0.7152 * channelToLinear(g) +
    0.0722 * channelToLinear(b)
  )
}

export function contrastRatio(hexA: string, hexB: string): number {
  const la = relativeLuminance(hexA)
  const lb = relativeLuminance(hexB)
  const [lighter, darker] = la >= lb ? [la, lb] : [lb, la]
  return (lighter + 0.05) / (darker + 0.05)
}

/** The foreground (white or dark ink) with the higher contrast on the given fill color. */
export function contrastOn(hex: string): string {
  return contrastRatio(hex, LIGHT_FOREGROUND) >=
    contrastRatio(hex, DARK_FOREGROUND)
    ? LIGHT_FOREGROUND
    : DARK_FOREGROUND
}
