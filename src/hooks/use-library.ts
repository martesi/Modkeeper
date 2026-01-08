import { useState, useEffect, useCallback } from 'react'
import { api } from '@/lib/api'
import type { LibraryDTO } from '@gen/bindings'

export function useLibrary() {
  const [library, setLibrary] = useState<LibraryDTO | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const fetchLibrary = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const data = await api.getLibrary()
      setLibrary(data)
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to load library'))
      setLibrary(null)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchLibrary()
  }, [fetchLibrary])

  const refresh = useCallback(() => {
    return fetchLibrary()
  }, [fetchLibrary])

  return {
    library,
    loading,
    error,
    refresh,
  }
}
