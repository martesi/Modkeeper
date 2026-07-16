import { cn } from '@/lib/utils'
import type { ThemeMode } from '@gen/bindings'
import { settingsText } from '../i18n/settings-text'

/**
 * Segmented System/Light/Dark control (reference Modkeeper.dc.html): uppercase segments in a soft
 * track; the active one is a white pill in BOTH themes (reference `themeOptions` style). Stable
 * dimensions — the selected state changes color only, never size.
 */
export function ThemeModeControl({
  value,
  onChange,
}: {
  value: ThemeMode
  onChange: (mode: ThemeMode) => void
}) {
  const options: { mode: ThemeMode; label: string }[] = [
    { mode: 'system', label: settingsText.themeSystem() },
    { mode: 'light', label: settingsText.themeLight() },
    { mode: 'dark', label: settingsText.themeDark() },
  ]

  return (
    <div
      role="radiogroup"
      aria-label={settingsText.appearance()}
      className="inline-flex gap-0.5 rounded-[0.875rem] border border-border bg-secondary p-1"
    >
      {options.map((option) => (
        <button
          key={option.mode}
          type="button"
          role="radio"
          aria-checked={value === option.mode}
          onClick={() => onChange(option.mode)}
          className={cn(
            'rounded-[0.625rem] px-4 py-2 text-[11px] font-extrabold uppercase tracking-[0.05em]',
            'outline-none transition-colors focus-visible:ring-[3px] focus-visible:ring-ring/50',
            value === option.mode
              ? // The reference pill is white regardless of theme (a deliberate literal).
                'bg-white text-neutral-900 shadow-sm'
              : 'text-muted-foreground hover:bg-accent',
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  )
}
