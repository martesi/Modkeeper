'use client'

import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { Toggle } from '@comps/toggle'
import { Link } from '@tanstack/react-router'
import type { Mod } from '@gen/bindings'
import { Trash2, Package, ChevronRight } from 'lucide-react'
import { useState, useEffect } from 'react'
import { msg, t } from '@lingui/core/macro'
import { DIVIDER } from '@/utils/constants'

interface ModCardProps {
  mod: Mod
  onToggle?: (id: string, isActive: boolean) => void
  onRemove?: (id: string) => void
  isSelectionMode?: boolean
  isSelected?: boolean
  onSelect?: () => void
}

export function ModCard({
  mod,
  onToggle,
  onRemove,
  isSelectionMode,
  isSelected,
  onSelect,
}: ModCardProps) {
  // Optimistic state for toggle
  const [optimisticActive, setOptimisticActive] = useState(mod.is_active)

  // Sync with prop changes
  useEffect(() => {
    setOptimisticActive(mod.is_active)
  }, [mod.is_active])

  const handleToggle = () => {
    const newState = !optimisticActive
    setOptimisticActive(newState) // Immediate UI update
    onToggle?.(mod.id, newState) // Fire and forget (non-blocking)
  }

  const handleRemove = () => {
    if (confirm(`Are you sure you want to remove "${mod.name}"?`)) {
      onRemove?.(mod.id)
    }
  }

  const handleCardClick = (e: React.MouseEvent) => {
    // Only trigger selection if clicking the card itself (not children)
    if (isSelectionMode && e.target === e.currentTarget) {
      onSelect?.()
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

  // Format author for display
  const authorDisplay = mod.manifest?.author
    ? Array.isArray(mod.manifest.author)
      ? mod.manifest.author.join(DIVIDER)
      : mod.manifest.author
    : null

  return (
    <div
      onClick={handleCardClick}
      className={`rounded-lg p-4 transition-all cursor-pointer ${
        isSelected ? 'border-2 border-primary bg-card' : 'border bg-card'
      } border-primary/20`}
    >
      {/* Header: Icon, Name Link, Remove Button */}
      <div className="flex items-cen justify-between mb-2">
        <Link
          to="/mod/$id"
          params={{ id: mod.id }}
          className="flex items-center gap-2 flex-1 min-w-0 hover:opacity-70 transition-opacity"
        >
          <Package className="size-5 text-muted-foreground shrink-0" />
          <div className="flex-1 min-w-0">
            <h3 className="font-semibold truncate">{mod.name}</h3>
            {/* Version and Author inline */}
            {mod.manifest && (mod.manifest.version || authorDisplay) && (
              <div className="text-xs text-muted-foreground truncate">
                {mod.manifest.version && (
                  <span>
                    <Trans>Version</Trans> {mod.manifest.version}
                  </span>
                )}
                {mod.manifest.version && authorDisplay && <span> • </span>}
                {authorDisplay && (
                  <span className="truncate">{authorDisplay}</span>
                )}
              </div>
            )}
          </div>
          <ChevronRight className="size-4 text-muted-foreground shrink-0" />
        </Link>
        <Button
          variant="ghost"
          size="icon"
          className="size-6 shrink-0 ml-2"
          onClick={handleRemove}
        >
          <Trash2 className="size-4" />
        </Button>
      </div>

      {/* Description with max height and ellipsis */}
      {mod.manifest?.description && (
        <div className="text-sm text-muted-foreground mb-3 pl-7 line-clamp-3">
          {mod.manifest.description}
        </div>
      )}

      {/* Footer: Type Badge and Toggle */}
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

        <Toggle
          pressed={optimisticActive}
          onPressedChange={handleToggle}
          variant="outline"
          size="sm"
          className="gap-1.5"
        >
          {optimisticActive ? <Trans>Active</Trans> : <Trans>Inactive</Trans>}
        </Toggle>
      </div>
    </div>
  )
}
