import { useState, useCallback } from 'react'
import { commands } from '@gen/bindings'
import { unwrapResult } from '@/lib/result'
import type { LibraryDTO } from '@gen/bindings'

export function useMods(
  library: LibraryDTO | null,
  onUpdate?: (library: LibraryDTO) => void,
) {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  const addMods = useCallback(
    async (paths: string[], unknownModName: string) => {
      try {
        setLoading(true)
        setError(null)
        const result = await unwrapResult(commands.addMods(paths, unknownModName))
        onUpdate?.(result)
        return result
      } catch (err) {
        const error =
          err instanceof Error ? err : new Error('Failed to add mods')
        setError(error)
        throw error
      } finally {
        setLoading(false)
      }
    },
    [onUpdate],
  )

  const removeMods = useCallback(
    async (ids: string[]) => {
      try {
        setLoading(true)
        setError(null)
        const result = await unwrapResult(commands.removeMods(ids))
        onUpdate?.(result)
        return result
      } catch (err) {
        const error =
          err instanceof Error ? err : new Error('Failed to remove mods')
        setError(error)
        throw error
      } finally {
        setLoading(false)
      }
    },
    [onUpdate],
  )

  const toggleMod = useCallback(
    async (id: string, isActive: boolean) => {
      try {
        setLoading(true)
        setError(null)
        const result = await unwrapResult(commands.toggleMod(id, isActive))
        onUpdate?.(result)
        return result
      } catch (err) {
        const error =
          err instanceof Error ? err : new Error('Failed to toggle mod')
        setError(error)
        throw error
      } finally {
        setLoading(false)
      }
    },
    [onUpdate],
  )

  const syncMods = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const result = await unwrapResult(commands.syncMods())
      onUpdate?.(result)
      return result
    } catch (err) {
      const error =
        err instanceof Error ? err : new Error('Failed to sync mods')
      setError(error)
      throw error
    } finally {
      setLoading(false)
    }
  }, [onUpdate])

  return {
    loading,
    error,
    addMods,
    removeMods,
    toggleMod,
    syncMods,
  }
}
