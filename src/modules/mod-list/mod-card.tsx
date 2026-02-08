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
import { checkDependencies, DependencyStatus } from '@/utils/dependency-check'
import {
  FolderSearch,
  Package,
  Trash2,
  AlertTriangle,
  AlertCircle,
} from 'lucide-react'
import { ett } from '@/utils/error'
import { ur } from '@/utils/result'
import { ModVersion } from '../../components/mod-version'
import { Tooltip, TooltipContent, TooltipTrigger } from '@comps/tooltip'

interface ModCardProps {
  mod: Mod
  allMods: Record<string, Mod | undefined>
}

export function ModCard({ mod, allMods }: ModCardProps) {
  const { toggle, remove } = useLibrary()
  const [open, { setTrue, set }] = useBoolean()

  const dependencies = useMemo(() => {
    return checkDependencies(mod, allMods)
  }, [mod, allMods])

  const hasIssues = dependencies.length > 0
  const hasMissing = dependencies.some(
    (d) => d.status === DependencyStatus.Missing,
  )

  const handleRevealMod = () => {
    commands.revealMod(mod.id).then(ur).catch(ett)
  }

  return (
    <Item className="mb-3 bg-card">
      <ItemMedia variant={mod.iconData ? 'image' : 'icon'}>
        {mod.iconData ? (
          <img
            src={mod.iconData}
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
          style={{ display: mod.iconData ? 'none' : 'block' }}
        />
      </ItemMedia>
      <ItemContent className="min-w-0">
        <ItemTitle className="w-full min-w-0">
          <div className="flex items-center gap-2 min-w-0">
            <Link
              to="/library/$id"
              params={{ id: mod.id }}
              className="hover:opacity-70 transition-opacity truncate"
            >
              {mod.name}
            </Link>
            {hasIssues && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="cursor-help">
                    {hasMissing ? (
                      <AlertCircle className="size-4 text-destructive" />
                    ) : (
                      <AlertTriangle className="size-4 text-yellow-500" />
                    )}
                  </div>
                </TooltipTrigger>
                <TooltipContent>
                  <div className="space-y-1">
                    <p className="font-semibold">
                      <Trans>Dependency Issues:</Trans>
                    </p>
                    {dependencies.map((dep) => {
                      if (dep.status === DependencyStatus.Satisfied) return null
                      return (
                        <p key={dep.id} className="text-xs">
                          <span className="font-mono">{dep.id}</span>:{' '}
                          {dep.status === DependencyStatus.Missing ? (
                            <Trans>
                              Missing (requires {dep.requiredVersion})
                            </Trans>
                          ) : (
                            <Trans>
                              Mismatch (requires {dep.requiredVersion}, found{' '}
                              {dep.foundVersion})
                            </Trans>
                          )}
                        </p>
                      )
                    })}
                  </div>
                </TooltipContent>
              </Tooltip>
            )}
          </div>
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
            checked={mod.isActive}
            onCheckedChange={(checked) => {
              toggle(mod.id, checked)
            }}
          />
        </div>
      </ItemActions>
    </Item>
  )
}
