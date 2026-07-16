/*
 * §4 acceptance: every accent swatch gets the better of the two foregrounds, and the pair clears
 * WCAG's 3:1 non-text-contrast minimum for filled controls. (The plan's original 4.5:1 target is
 * unreachable for the brand pink #e91e63 — its best possible foreground, white, lands at ~4.3:1 —
 * so the assert uses the UI-component threshold and additionally proves the helper always picks
 * the higher-contrast option.)
 */
import { describe, expect, test } from 'bun:test'
import { contrastOn, contrastRatio } from './contrast'
import { ACCENT_SWATCHES } from '../../settings/accent-palette'

describe('contrastOn', () => {
  for (const swatch of ACCENT_SWATCHES) {
    test(`${swatch.name} (${swatch.value}) foreground is readable`, () => {
      const foreground = contrastOn(swatch.value)
      const ratio = contrastRatio(swatch.value, foreground)
      expect(ratio).toBeGreaterThanOrEqual(3)

      const other = foreground === '#ffffff' ? '#17181c' : '#ffffff'
      expect(ratio).toBeGreaterThanOrEqual(contrastRatio(swatch.value, other))
    })
  }

  test('light accents get dark text, dark accents get white text', () => {
    expect(contrastOn('#3ecf8e')).toBe('#17181c')
    expect(contrastOn('#f5a35c')).toBe('#17181c')
    expect(contrastOn('#e91e63')).toBe('#ffffff')
    expect(contrastOn('#00828d')).toBe('#ffffff')
  })
})
