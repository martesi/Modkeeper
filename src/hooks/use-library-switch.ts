import { useState, useEffect, useCallback } from 'react'
import { commands } from '@gen/bindings'
import { unwrapResult } from '@/lib/result'
import type { LibrarySwitch, LibraryCreationRequirement } from '@gen/bindings'

export function useLibrarySwitch() {
  const [librarySwitch, setLibrarySwitch] = useState<LibrarySwitch | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const fetchLibrarySwitch = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      // Try to get the current library - if it exists, we can infer the switch state
      // Otherwise, we'll have an empty state
      try {
        const library = await unwrapResult(commands.getLibrary())
        // If we have a library, create a minimal LibrarySwitch
        // Note: This doesn't include all known libraries, but gives us the active one
        setLibrarySwitch({
          active: library,
          libraries: [library],
        })
      } catch {
        // No active library
        setLibrarySwitch({
          active: null,
          libraries: [],
        })
      }
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error('Failed to load libraries'),
      )
      setLibrarySwitch(null)
    } finally {
      setLoading(false)
    }
  }, [])

  const openLibrary = useCallback(async (path: string) => {
    try {
      setLoading(true)
      setError(null)
      const result = await unwrapResult(commands.openLibrary(path))
      // The result is LibrarySwitch, update state accordingly
      setLibrarySwitch(result)
      return result
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to open library'))
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const createLibrary = useCallback(
    async (requirement: LibraryCreationRequirement) => {
      try {
        setLoading(true)
        setError(null)
        const result = await unwrapResult(commands.createLibrary(requirement))
        // The result is LibrarySwitch, update state accordingly
        setLibrarySwitch(result)
        return result
      } catch (err) {
        setError(
          err instanceof Error ? err : new Error('Failed to create library'),
        )
        throw err
      } finally {
        setLoading(false)
      }
    },
    [],
  )

  const renameLibrary = useCallback(async (name: string) => {
    try {
      setLoading(true)
      setError(null)
      const result = await unwrapResult(commands.renameLibrary(name))
      setLibrarySwitch(result)
      return result
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error('Failed to rename library'),
      )
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const closeLibrary = useCallback(async (repoRoot: string) => {
    try {
      setLoading(true)
      setError(null)
      const result = await unwrapResult(commands.closeLibrary(repoRoot))
      setLibrarySwitch(result)
      return result
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error('Failed to close library'),
      )
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const removeLibrary = useCallback(async (repoRoot: string) => {
    try {
      setLoading(true)
      setError(null)
      const result = await unwrapResult(commands.removeLibrary(repoRoot))
      setLibrarySwitch(result)
      return result
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error('Failed to remove library'),
      )
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchLibrarySwitch()
  }, [fetchLibrarySwitch])

  return {
    librarySwitch,
    loading,
    error,
    openLibrary,
    createLibrary,
    renameLibrary,
    closeLibrary,
    removeLibrary,
    refresh: fetchLibrarySwitch,
  }
}
