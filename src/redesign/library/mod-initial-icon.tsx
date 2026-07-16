import type { ModType } from '../data/redesign-types'

/**
 * The reference card's tinted initial tile (Modkeeper.dc.html `mod.iconStyle`). The reference
 * seeds a color per mod, but ModSummary carries none (C6) — the tint is keyed to the ModType via
 * the chart tokens instead, so type stays visually scannable.
 */
const TINT_BY_TYPE: Record<ModType, string> = {
  client: 'var(--chart-2)',
  server: 'var(--chart-1)',
  both: 'var(--chart-3)',
  unknown: 'var(--muted-foreground)',
}

export function ModInitialIcon({
  name,
  type,
}: {
  name: string
  type: ModType
}) {
  const tint = TINT_BY_TYPE[type]
  return (
    <span
      className="inline-flex size-11 shrink-0 items-center justify-center rounded-[0.875rem] font-heading text-base font-extrabold"
      style={{
        backgroundColor: `color-mix(in srgb, ${tint} 13%, transparent)`,
        color: tint,
      }}
      aria-hidden
    >
      {(name[0] ?? '?').toUpperCase()}
    </span>
  )
}
