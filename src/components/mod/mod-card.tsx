'use client'

import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { Toggle } from '@comps/toggle'
import { Link } from '@tanstack/react-router'
import type { Mod } from '@gen/bindings'
import { Trash2, Package, ChevronRight } from 'lucide-react'
import { useState, useEffect } from 'react'
import { DIVIDER } from '@/utils/translation'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@comps/alert-dialog'

interface ModCardProps {
  mod: Mod
  onToggle?: (id: string, isActive: boolean) => void
  onRemove?: (id: string) => void
}

export function ModCard({ mod, onToggle, onRemove }: ModCardProps) {
  // Optimistic state for toggle
  const [optimisticActive, setOptimisticActive] = useState(mod.is_active)
  const [showRemoveDialog, setShowRemoveDialog] = useState(false)

  // Sync with prop changes
  useEffect(() => {
    setOptimisticActive(mod.is_active)
  }, [mod.is_active])

  const handleToggle = () => {
    const newState = !optimisticActive
    setOptimisticActive(newState) // Immediate UI update
    onToggle?.(mod.id, newState) // Fire and forget (non-blocking)
  }

  const handleRemoveClick = () => {
    setShowRemoveDialog(true)
  }

  const handleRemoveConfirm = () => {
    setShowRemoveDialog(false)
    onRemove?.(mod.id)
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
      ? mod.manifest.author.join(DIVIDER())
      : mod.manifest.author
    : null

  return (
    <div className="rounded-lg p-4 border bg-card border-primary/20 flex flex-col h-full">
      {/* Header: Icon, Name Link, Remove Button */}
      <div className="flex items-cen justify-between mb-2">
        <Link
          to="/$id"
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
          onClick={handleRemoveClick}
        >
          <Trash2 className="size-4" />
        </Button>
      </div>

      <AlertDialog open={showRemoveDialog} onOpenChange={setShowRemoveDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Remove Mod</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              <Trans>
                Are you sure you want to remove "{mod.name}"? This action cannot
                be undone.
              </Trans>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleRemoveConfirm}
              variant="destructive"
            >
              <Trans>Remove</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Description with fixed height */}
      {mod.manifest?.description && (
        <div className="text-sm text-muted-foreground mb-3 pl-7 h-16 overflow-hidden shrink-0">
          {mod.manifest.description}
        </div>
      )}

      {/* Footer: Type Badge and Toggle */}
      <div className="flex items-center justify-between mt-auto">
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
