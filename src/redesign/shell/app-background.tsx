/*
 * Full-viewport app background (consolidated-spec.md §12): warm base surface with two soft brand
 * tints so the glass panels have something to blur. Fixed and non-interactive; sits behind
 * everything.
 */
export function AppBackground() {
  return (
    <div
      aria-hidden
      className="fixed inset-0 -z-10 overflow-hidden bg-[var(--mk-surface)]"
    >
      <div
        className="absolute -left-32 -top-40 size-[28rem] rounded-full blur-3xl"
        style={{ backgroundColor: 'var(--mk-state-hover)' }}
      />
      <div
        className="absolute -bottom-48 -right-32 size-[32rem] rounded-full blur-3xl"
        style={{ backgroundColor: 'rgba(0, 130, 141, 0.08)' }}
      />
    </div>
  )
}
