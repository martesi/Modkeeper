'use client'

import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import type { Mod } from '@gen/bindings'
import { Trash2, Package } from 'lucide-react'

interface ModCardProps {
  mod: Mod
  onToggle?: (id: string, isActive: boolean) => void
  onRemove?: (id: string) => void
}

export function ModCard({ mod, onToggle, onRemove }: ModCardProps) {
  const handleToggle = () => {
    onToggle?.(mod.id, !mod.is_active)
  }

  const handleRemove = () => {
    if (confirm(`Are you sure you want to remove "${mod.name}"?`)) {
      onRemove?.(mod.id)
    }
  }

  const getModTypeLabel = () => {
    switch (mod.mod_type) {
      case 'Client':
        return <Trans>Client</Trans>
      case 'Server':
        return <Trans>Server</Trans>
      case 'Both':
        return <Trans>Both</Trans>
      default:
        return <Trans>Unknown</Trans>
    }
  }

  return (
    <div
      className={`rounded-lg border p-4 transition-colors ${
        mod.is_active ? 'bg-card border-primary/50' : 'bg-muted/50 border-muted'
      }`}
    >
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2">
          <Package className="size-5 text-muted-foreground" />
          <h3 className="font-semibold">{mod.name}</h3>
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="size-6"
          onClick={handleRemove}
        >
          <Trash2 className="size-4" />
        </Button>
      </div>

      {mod.manifest && (
        <div className="text-sm text-muted-foreground mb-2 space-y-1">
          {mod.manifest.version && (
            <div>
              <Trans>Version: {mod.manifest.version}</Trans>
            </div>
          )}
          {mod.manifest.author && (
            <div>
              <Trans>Author: {mod.manifest.author}</Trans>
            </div>
          )}
        </div>
      )}

      <div className="flex items-center justify-between mt-4">
        <span
          className={`text-xs px-2 py-1 rounded ${
            mod.mod_type === 'Client'
              ? 'bg-blue-500/20 text-blue-700 dark:text-blue-400'
              : mod.mod_type === 'Server'
                ? 'bg-green-500/20 text-green-700 dark:text-green-400'
                : mod.mod_type === 'Both'
                  ? 'bg-purple-500/20 text-purple-700 dark:text-purple-400'
                  : 'bg-gray-500/20 text-gray-700 dark:text-gray-400'
          }`}
        >
          {getModTypeLabel()}
        </span>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={mod.is_active}
            onChange={handleToggle}
            className="rounded"
          />
          <span className="text-sm">
            {mod.is_active ? <Trans>Active</Trans> : <Trans>Inactive</Trans>}
          </span>
        </label>
      </div>
    </div>
  )
}
