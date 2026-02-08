import { Button } from '@comps/button'
import { Package, Trash2, FolderOpen } from 'lucide-react'
import { Trans } from '@lingui/react/macro'
import { Switch } from '@comps/switch'
import { commands, Mod } from '@gen/bindings'
import { ur } from '@/utils/result'
import { ConfirmPopover } from '@comps/confirm-popover'
import { useState } from 'react'
import { ModVersion } from 'src/components/mod-version'

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
    <div className="bg-card rounded-xl p-6 shadow-sm border grid grid-cols-[auto_1fr] items-center gap-6">
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
      <div className="min-w-0 flex flex-col gap-2">
        <div className="min-w-0 w-full">
          <h1 className="text-2xl font-bold truncate mb-1" title={mod.name}>
            {mod.name}
          </h1>
          <ModVersion mod={mod} />
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <Switch checked={mod.isActive} onCheckedChange={onToggle} />

          <Button variant="ghost" size="icon" onClick={handleReveal}>
            <FolderOpen className="size-4" />
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
                size="icon"
                className="text-destructive hover:text-destructive hover:bg-destructive/10"
              >
                <Trash2 className="size-4" />
              </Button>
            }
          />
        </div>
      </div>
    </div>
  )
}
