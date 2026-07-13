import { cn } from '@/lib/utils'
import type { ThemeMode } from '@gen/bindings'
import { settingsText } from '../i18n/settings-text'

/**
 * Segmented System/Light/Dark control (consolidated-spec.md §12.6). Stable dimensions — the
 * selected state changes color only, never size.
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
      className="inline-flex rounded-[var(--mk-radius-control)] border border-[var(--mk-outline)] bg-[var(--mk-surface)] p-1"
    >
      {options.map((option) => (
        <button
          key={option.mode}
          type="button"
          role="radio"
          aria-checked={value === option.mode}
          onClick={() => onChange(option.mode)}
          className={cn(
            'mk-focus-ring h-8 rounded-[calc(var(--mk-radius-control)-0.25rem)] px-3 text-sm transition-colors',
            value === option.mode
              ? 'bg-[var(--mk-primary)] font-medium text-[var(--mk-on-primary)]'
              : 'text-[var(--mk-text-muted)] hover:bg-[var(--mk-state-hover)]',
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  )
}
