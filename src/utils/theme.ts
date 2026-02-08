/**
 * Converts a theme string to an isDark boolean for window effect API
 * @param theme - The theme string from next-themes: 'dark', 'light', or 'system'
 * @returns true for dark, false for light, undefined for system
 */
export function themeToIsDark(theme: string | undefined): boolean | undefined {
  if (theme === 'dark') return true
  if (theme === 'light') return false
  return undefined
}
