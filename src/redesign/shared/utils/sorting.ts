/*
 * Shared sort helpers (consolidated-spec.md §10 shared/utils). Locale-aware, numeric-friendly
 * ("Mod 2" before "Mod 10"), so the grid's name sort matches what a person expects.
 */

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: 'base',
})

export function compareByName(
  a: { name: string },
  b: { name: string },
): number {
  return collator.compare(a.name, b.name)
}
