import { cn } from '@/lib/utils'
import { settingsText } from '../i18n/settings-text'

// One shipped locale today; the control exists so adding a catalog is a one-line change here.
const LOCALES: { value: string; label: string }[] = [
  { value: 'en-US', label: 'English (US)' },
]

/** Language select (consolidated-spec.md §12.6): saving triggers `changeLocale` via the applier. */
export function LanguageSelect({
  value,
  onChange,
}: {
  value: string
  onChange: (language: string) => void
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      aria-label={settingsText.language()}
      className={cn(
        'mk-focus-ring h-10 rounded-[var(--mk-radius-control)] px-3 text-sm',
        'border border-[var(--mk-outline)] bg-[var(--mk-surface)] text-[var(--mk-text)]',
      )}
    >
      {LOCALES.map((locale) => (
        <option key={locale.value} value={locale.value}>
          {locale.label}
        </option>
      ))}
    </select>
  )
}
