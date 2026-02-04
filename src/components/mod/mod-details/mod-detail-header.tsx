import { Button } from '@comps/button'
import { Package, Trash2, FolderOpen } from 'lucide-react'
import { Trans } from '@lingui/react/macro'
import { Switch } from '@comps/switch'
import { commands, Mod } from '@gen/bindings'
import { ur } from '@/utils/result'
import { ConfirmPopover } from '@comps/confirm-popover'
import { useState } from 'react'

interface ModDetailHeaderProps {
  mod: Mod
  onToggle: () => void
  onRemove: () => void
}

export function ModDetailHeader({
  mod,
  onToggle,
  onRemove,
}: ModDetailHeaderProps) {
  const [open, setOpen] = useState(false)

  const handleReveal = () => {
    ur(commands.revealMod(mod.id))
  }

  return (
    <div className="bg-card rounded-xl p-6 shadow-sm border flex items-center gap-6">
      {/* Icon */}
      <div className="shrink-0">
        {mod.iconData ? (
          <img
            src={mod.iconData}
            alt={mod.name}
            className="size-20 rounded-lg object-cover bg-muted"
          />
        ) : (
          <div className="size-20 rounded-lg bg-muted flex items-center justify-center">
            <Package className="size-10 text-muted-foreground" />
          </div>
        )}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <h1 className="text-2xl font-bold truncate mb-1">{mod.name}</h1>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          {mod.manifest?.author && (
            <span>
              <Trans>by</Trans>{' '}
              {typeof mod.manifest.author === 'string'
                ? mod.manifest.author
                : mod.manifest.author.join(', ')}
            </span>
          )}
          {mod.manifest?.version && (
            <>
              <span>•</span>
              <span>v{mod.manifest.version}</span>
            </>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 mr-4">
          <Switch checked={mod.isActive} onCheckedChange={onToggle} />
          <span className="text-sm font-medium">
            {mod.isActive ? <Trans>Enabled</Trans> : <Trans>Disabled</Trans>}
          </span>
        </div>

        <Button variant="outline" size="sm" onClick={handleReveal}>
          <FolderOpen className="size-4 mr-2" />
          <Trans>Reveal</Trans>
        </Button>

        <ConfirmPopover
          open={open}
          onOpenChange={setOpen}
          title={<Trans>Remove Mod</Trans>}
          description={
            <Trans>
              Are you sure you want to remove &quot;{mod.name}&quot;? This
              action cannot be undone.
            </Trans>
          }
          confirmLabel={<Trans>Remove</Trans>}
          variant="destructive"
          onConfirm={onRemove}
          trigger={
            <Button
              variant="ghost"
              size="sm"
              className="text-destructive hover:text-destructive hover:bg-destructive/10"
            >
              <Trash2 className="size-4 mr-2" />
              <Trans>Remove</Trans>
            </Button>
          }
        />
      </div>
    </div>
  )
}
