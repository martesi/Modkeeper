import { msg, t } from '@lingui/core/macro'
import { translateError } from './error'
import type { SError } from '@gen/bindings'

/**
 * Adds a library by prompting the user to select a game root directory.
 * The backend automatically:
 * - Derives repo_root from game_root as game_root/.mod_keeper
 * - If a library already exists at that location and is valid, opens it
 * - If no library exists, creates a new one
 * @param createLibrary - Function to add the library (from useLibrarySwitch hook)
 */
export async function addLibraryFromDialog(
  createLibrary: (requirement: {
    name: string
    game_root: string
    repo_root: string
  }) => Promise<unknown>,
): Promise<void> {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: t(msg`Select Game Root Directory`),
    })

    // Ignore if no path received (user cancelled)
    if (!selected || typeof selected !== 'string') {
      return
    }

    try {
      // Use translated "Unnamed Library" as the library name
      const libraryName = t(msg`Unnamed Library`)
      // Backend will derive repo_root from game_root, but we need to provide it for the type
      // The backend always uses game_root/.mod_keeper regardless of what we pass
      const separator = selected.includes('\\') ? '\\' : '/'
      const repoRoot = `${selected}${separator}.mod_keeper`

      await createLibrary({
        name: libraryName,
        game_root: selected,
        repo_root: repoRoot,
      })
    } catch (err) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : translateError(err as SError) || 'Failed to add library'
      // Show error to user - you might want to use a toast notification here
      console.error('Failed to add library:', errorMessage)
      throw new Error(errorMessage)
    }
  } catch (err) {
    // Re-throw if it's already an Error with message
    if (err instanceof Error && err.message !== 'Failed to add library') {
      throw err
    }
    // User cancelled or error opening dialog - silently return
    return
  }
}
