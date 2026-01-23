'use client'

import { useLibrary } from '@/hooks/use-library'
import { Button } from '@comps/button'
import { Switch } from '@comps/switch'
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemMedia,
  ItemTitle,
} from '@comps/item'
import { ConfirmPopover } from '@comps/confirm-popover'
import type { Mod } from '@gen/bindings'
import { commands } from '@gen/bindings'
import { Trans } from '@lingui/react/macro'
import { Link } from '@tanstack/react-router'
import { useBoolean } from 'ahooks'
import { FolderSearch, Package, Trash2 } from 'lucide-react'
import { ModVersion } from './mod-version'

interface ModCardProps {
  mod: Mod
}

export function ModCard ({ mod }: ModCardProps) {
  const { toggle, remove } = useLibrary()
  const [open, { setTrue, set }] = useBoolean()

  const handleRevealMod = async () => {
    try {
      const result = await commands.revealMod(mod.id)
      if (result.status === 'error') {
        console.error('Failed to reveal mod:', result.error)
      }
    } catch (err) {
      console.error('Failed to reveal mod:', err)
    }
  }

  return (
    <Item variant="outline" className="mb-3">
      <ItemMedia variant={mod.icon_data ? 'image' : 'icon'}>
        {mod.icon_data ? (
          <img
            src={mod.icon_data}
            alt={mod.name}
            onError={(e) => {
              // Fallback to Package icon on error
              e.currentTarget.style.display = 'none'
              const fallback = e.currentTarget.nextElementSibling as HTMLElement
              if (fallback) fallback.style.display = 'block'
            }}
          />
        ) : null}
        <Package
          className="size-5 text-muted-foreground"
          style={{ display: mod.icon_data ? 'none' : 'block' }}
        />
      </ItemMedia>
      <ItemContent>
        <ItemTitle>
          <Link
            to="/library/$id"
            params={{ id: mod.id }}
            className="hover:opacity-70 transition-opacity"
          >
            {mod.name}
          </Link>
        </ItemTitle>
        <ItemDescription>
          <ModVersion mod={mod} />
        </ItemDescription>
      </ItemContent>
      <ItemActions>
        <div className="flex items-center gap-2">
          <ConfirmPopover
            open={open}
            onOpenChange={set}
            title={<Trans>Remove Mod</Trans>}
            description={
              <Trans>
                Are you sure you want to remove &quot;{mod.name}&quot;? This
                action cannot be undone.
              </Trans>
            }
            confirmLabel={<Trans>Remove</Trans>}
            variant="destructive"
            onConfirm={() => remove([mod.id])}
            trigger={
              <Button
                variant="ghost"
                size="icon"
                className="size-6"
                onClick={setTrue}
              >
                <Trash2 className="size-4" />
              </Button>
            }
            side="top"
          />
          <Button
            variant="ghost"
            size="icon"
            className="size-6"
            onClick={handleRevealMod}
          >
            <FolderSearch className="size-4" />
          </Button>
          <Switch
            checked={mod.is_active}
            onCheckedChange={(checked) => {
              toggle(mod.id, checked)
            }}
          />
        </div>
      </ItemActions>
    </Item>
  )
}
