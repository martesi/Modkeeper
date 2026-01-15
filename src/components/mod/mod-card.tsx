'use client'

import { ModTypeBadge } from '@/components/mod/mod-type-badge'
import { useLibrary } from '@/hooks/use-library'
import { Button } from '@comps/button'
import { Switch } from '@comps/switch'
import type { Mod } from '@gen/bindings'
import { Trans } from '@lingui/react/macro'
import { Link } from '@tanstack/react-router'
import { useBoolean } from 'ahooks'
import { ChevronRight, Package, Trash2 } from 'lucide-react'
import { RemoveModDialog } from '../dialog/remove-mod'
import { ModVersion } from './mod-version'

interface ModCardProps {
  mod: Mod
}

export function ModCard({ mod }: ModCardProps) {
  const { toggle, remove } = useLibrary()
  const [open, { setTrue, set }] = useBoolean()

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
            <ModVersion className="text-xs" mod={mod} />
          </div>
          <ChevronRight className="size-4 text-muted-foreground shrink-0" />
        </Link>
        <Button
          variant="ghost"
          size="icon"
          className="size-6 shrink-0 ml-2"
          onClick={setTrue}
        >
          <Trash2 className="size-4" />
        </Button>
      </div>

      <RemoveModDialog
        mod={mod}
        open={open}
        setOpen={set}
        onConfirm={() => remove([mod.id])}
      />

      {/* Description with fixed height */}
      {mod.manifest?.description && (
        <div className="text-sm text-muted-foreground mb-3 pl-7 h-16 overflow-hidden shrink-0">
          {mod.manifest.description}
        </div>
      )}

      {/* Footer: Type Badge and Switch */}
      <div className="flex items-center justify-between mt-auto">
        <div className="flex gap-2">
          <ModTypeBadge type={mod.mod_type} />
        </div>

        <div className="flex items-center gap-2">
          <span
            className={`text-xs font-medium ${
              mod.is_active
                ? 'text-green-600 dark:text-green-400'
                : 'text-muted-foreground'
            }`}
          >
            {mod.is_active ? <Trans>Active</Trans> : <Trans>Inactive</Trans>}
          </span>
          <Switch
            checked={mod.is_active}
            onCheckedChange={() => toggle(mod.id, !mod.is_active)}
          />
        </div>
      </div>
    </div>
  )
}
