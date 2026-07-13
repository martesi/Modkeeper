import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'
import { DEFAULT_ACCENT } from '../data/settings-repository'
import { settingsText } from '../i18n/settings-text'

// Fidelity pink first (the default), then warm-compatible alternatives (spec §11 guardrails).
const SWATCHES: { name: string; value: string }[] = [
  { name: 'Pink', value: DEFAULT_ACCENT },
  { name: 'Teal', value: '#00828d' },
  { name: 'Purple', value: '#8b5cf6' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Green', value: '#10b981' },
  { name: 'Blue', value: '#3b82f6' },
]

/** Accent swatch strip (consolidated-spec.md §12.6): applies immediately and persists. */
export function AccentSwatches({
  value,
  onChange,
}: {
  value: string
  onChange: (color: string) => void
}) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      {SWATCHES.map((swatch) => {
        const selected = value.toLowerCase() === swatch.value.toLowerCase()
        return (
          <button
            key={swatch.value}
            type="button"
            aria-label={settingsText.accentSwatch(swatch.name)}
            aria-pressed={selected}
            title={swatch.name}
            onClick={() => onChange(swatch.value)}
            className={cn(
              'mk-focus-ring inline-flex size-8 items-center justify-center rounded-full border-2 transition-transform',
              selected
                ? 'scale-110 border-[var(--mk-text)]'
                : 'border-transparent hover:scale-105',
            )}
            style={{ backgroundColor: swatch.value }}
          >
            {selected && <Check className="size-4 text-white" aria-hidden />}
          </button>
        )
      })}
    </div>
  )
}
