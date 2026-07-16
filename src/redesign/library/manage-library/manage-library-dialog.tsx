import { useState } from 'react'
import { useAtomValue } from 'jotai'
import { isTauri } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import {
  Copy,
  FolderOpen,
  Play,
  Plus,
  RefreshCw,
  Settings2,
  TriangleAlert,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { FidelityButton } from '../../shared/components/fidelity-button'
import { FidelityIconButton } from '../../shared/components/fidelity-icon-button'
import { FidelityInput } from '../../shared/components/fidelity-input'
import {
  activeLibraryIdAtom,
  libraryListAtom,
  libraryWorkspaceAtom,
  pendingTasksAtom,
} from '../../state/library-state'
import {
  activateLibrary,
  createLibrary,
  deleteLibrary,
  rebuildLibraryCache,
  renameLibrary,
} from '../../data/library-repository'
import { launchTool } from '../../data/tools-repository'
import { isLibrarySummary } from '../../data/redesign-types'
import type {
  LibraryEntry,
  LibraryStub,
  LibrarySummary,
  ToolSummary,
} from '../../data/redesign-types'
import { openGameRoot, openLibraryRoot } from '../../shared/utils/app-opener'
import { libraryText } from '../../i18n/library-text'
import { ToolIconGlyph } from '../tool-icon-glyph'
import { DeleteLibraryConfirmDialog } from './delete-library-confirm-dialog'
import { ConfigureToolDialog } from './configure-tool-dialog'

/**
 * Manage Library dialog (reference Modkeeper.dc.html): strong-glass body, profile tabs across
 * every registered entry (readable summaries AND path-only stubs, C13) plus a dashed plus tab
 * that picks a game folder and creates+selects. The body is the reference's two-column layout —
 * section labels left, controls right — covering identity, installation paths, and executable
 * tools. Rebuild Cache is fire-and-track and only valid for the ACTIVE library (§7e rejects
 * non-active libraryIds), so it is disabled elsewhere with a hint.
 */
export function ManageLibraryDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const libraries = useAtomValue(libraryListAtom)
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)
  const [selectedKey, setSelectedKey] = useState<string | null>(null)

  // Selection is resolved against the CURRENT list every render, so a deleted entry or a fresh
  // workspace can never leave a dangling tab selected.
  const selected =
    libraries.find((entry) => entryKey(entry) === selectedKey) ??
    libraries.find(
      (entry) => isLibrarySummary(entry) && entry.id === activeLibraryId,
    ) ??
    libraries[0] ??
    null

  async function handleAddLibrary() {
    const gameRoot = await pickGameRootFolder()
    if (!gameRoot) return
    const workspace = await createLibrary({ gameRoot })
    // create+select (§12.4): the created library becomes the active one.
    if (workspace.activeLibraryId) setSelectedKey(workspace.activeLibraryId)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="mk-glass-strong mk-scrollbar flex max-h-[88vh] w-[min(67.5rem,calc(100vw-2rem))] flex-col gap-0 overflow-y-auto rounded-[2rem] border-border bg-popover p-0 sm:max-w-none">
        <DialogHeader className="px-9 pb-1 pt-8">
          <DialogTitle className="font-heading text-[26px] font-extrabold">
            {libraryText.manageDialogTitle()}
          </DialogTitle>
          <DialogDescription className="text-sm">
            {libraryText.manageDialogDescription()}
          </DialogDescription>
        </DialogHeader>

        <div
          className="flex flex-wrap items-center gap-2.5 px-9 pb-1 pt-5"
          role="tablist"
        >
          {libraries.map((entry) => {
            const key = entryKey(entry)
            const isSelected = selected !== null && key === entryKey(selected)
            return (
              <button
                key={key}
                type="button"
                role="tab"
                aria-selected={isSelected}
                onClick={() => setSelectedKey(key)}
                className={cn(
                  'h-11 max-w-56 truncate rounded-2xl px-4.5 text-sm outline-none transition-colors focus-visible:ring-[3px] focus-visible:ring-ring/50',
                  isSelected
                    ? 'bg-primary font-bold text-primary-foreground'
                    : 'border border-border bg-secondary font-semibold text-foreground hover:bg-accent',
                )}
              >
                {isLibrarySummary(entry)
                  ? entry.name
                  : libraryText.unreadableLibrary()}
              </button>
            )
          })}
          <button
            type="button"
            onClick={() => void handleAddLibrary()}
            aria-label={libraryText.addLibraryTab()}
            title={libraryText.addLibraryTab()}
            className={cn(
              'inline-flex size-11 items-center justify-center rounded-2xl',
              'border-2 border-dashed border-border text-muted-foreground outline-none transition-colors',
              'hover:border-primary hover:text-primary focus-visible:ring-[3px] focus-visible:ring-ring/50',
            )}
          >
            <Plus className="size-4.5" aria-hidden />
          </button>
        </div>

        {selected === null ? null : isLibrarySummary(selected) ? (
          <LibraryDetails key={selected.id} library={selected} />
        ) : (
          <StubDetails stub={selected} />
        )}
      </DialogContent>
    </Dialog>
  )
}

function LibraryDetails({ library }: { library: LibrarySummary }) {
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)
  const pendingTasks = useAtomValue(pendingTasksAtom)
  const workspace = useAtomValue(libraryWorkspaceAtom)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [activating, setActivating] = useState(false)
  const [nameDraft, setNameDraft] = useState(library.name)
  const [renaming, setRenaming] = useState(false)
  const [configureTool, setConfigureTool] = useState<{
    tool: ToolSummary | null
  } | null>(null)

  const tools = workspace?.toolsByLibraryId[library.id] ?? []
  const isActive = library.id === activeLibraryId
  const busy = pendingTasks.some((task) => task.libraryId === library.id)
  const nameDirty =
    nameDraft.trim() !== library.name && nameDraft.trim() !== ''

  async function handleActivate() {
    setActivating(true)
    try {
      await activateLibrary(library.id)
    } finally {
      setActivating(false)
    }
  }

  async function handleRename() {
    setRenaming(true)
    try {
      await renameLibrary({ libraryId: library.id, name: nameDraft.trim() })
    } finally {
      setRenaming(false)
    }
  }

  return (
    <>
      <div className="grid gap-7 px-9 py-7 md:grid-cols-[16.25rem_1fr]">
        <div>
          <SectionLabel
            title={libraryText.identitySection()}
            description={
              library.sptVersion
                ? `${libraryText.identityDescription()} · ${libraryText.sptVersion(library.sptVersion)}`
                : libraryText.identityDescription()
            }
          />
          <SectionLabel
            title={libraryText.pathsSection()}
            description={libraryText.pathsDescription()}
          />
          <SectionLabel
            title={libraryText.toolsSection()}
            description={libraryText.toolsDescription()}
            last
          />
          <button
            type="button"
            onClick={() => setConfigureTool({ tool: null })}
            className="rounded-sm text-[13px] font-bold text-primary outline-none transition-opacity hover:opacity-80 focus-visible:ring-[3px] focus-visible:ring-ring/50"
          >
            {libraryText.registerNewTool()}
          </button>
        </div>

        <div className="min-w-0">
          <div className="mb-5 flex gap-2.5">
            <FidelityInput
              value={nameDraft}
              onChange={(event) => setNameDraft(event.target.value)}
              aria-label={libraryText.nameLabel()}
              className="font-semibold"
            />
            <FidelityButton
              disabled={!nameDirty}
              busy={renaming}
              onClick={() => void handleRename()}
            >
              {libraryText.saveName()}
            </FidelityButton>
          </div>

          <PathRow
            label={libraryText.gameRootLabel()}
            path={library.gameRoot}
            onOpen={() => void openGameRoot(library)}
          />
          <PathRow
            label={libraryText.libraryRootLabel()}
            path={library.libraryRoot}
            onOpen={() => void openLibraryRoot(library)}
          />

          <div className="mt-5 flex flex-col gap-3">
            {tools.map((tool) => (
              <ToolRow
                key={tool.id}
                tool={tool}
                onConfigure={() => setConfigureTool({ tool })}
              />
            ))}
          </div>
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3.5 border-t border-border px-9 py-5">
        <div className="flex items-center gap-2">
          <FidelityButton
            variant="ghost"
            size="sm"
            className="text-[13px] font-bold text-muted-foreground"
            disabled={!isActive}
            busy={busy}
            title={isActive ? undefined : libraryText.rebuildNeedsActive()}
            onClick={() => void rebuildLibraryCache(library.id)}
          >
            {!busy && <RefreshCw />}
            {libraryText.rebuildCache()}
          </FidelityButton>
          <FidelityButton
            variant="ghost"
            size="sm"
            className="text-[13px] font-bold text-destructive hover:text-destructive"
            onClick={() => setConfirmDelete(true)}
          >
            {libraryText.deleteLibrary()}
          </FidelityButton>
        </div>
        <FidelityButton
          size="lg"
          className="rounded-2xl px-7 text-[13px] font-extrabold uppercase tracking-[0.05em]"
          disabled={isActive}
          busy={activating}
          onClick={() => void handleActivate()}
        >
          {isActive ? libraryText.activated() : libraryText.activate()}
        </FidelityButton>
      </div>

      <DeleteLibraryConfirmDialog
        library={library}
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        onDeleted={() => {}}
      />
      <ConfigureToolDialog
        libraryId={library.id}
        tool={configureTool?.tool ?? null}
        open={configureTool !== null}
        onOpenChange={(next) => {
          if (!next) setConfigureTool(null)
        }}
      />
    </>
  )
}

function SectionLabel({
  title,
  description,
  last = false,
}: {
  title: string
  description: string
  last?: boolean
}) {
  return (
    <div className={last ? 'mb-2.5' : 'mb-7'}>
      <h3 className="font-heading mb-1.5 text-[15px] font-bold text-foreground">
        {title}
      </h3>
      <p className="text-[13px] text-muted-foreground">{description}</p>
    </div>
  )
}

function PathRow({
  label,
  path,
  onOpen,
}: {
  label: string
  path: string
  onOpen: () => void
}) {
  async function handleCopy() {
    await navigator.clipboard.writeText(path)
    toast.success(libraryText.pathCopied())
  }

  return (
    <div className="mb-3 flex items-center gap-3.5 rounded-2xl border border-border bg-secondary px-4.5 py-3.5">
      <span className="shrink-0 text-sm font-bold uppercase tracking-[0.02em] text-foreground">
        {label}
      </span>
      <span
        className="min-w-0 flex-1 truncate text-[13px] text-muted-foreground"
        title={path}
      >
        {path}
      </span>
      <FidelityIconButton
        size="sm"
        variant="secondary"
        className="rounded-[0.625rem] bg-transparent"
        aria-label={libraryText.copyPath()}
        title={libraryText.copyPath()}
        onClick={() => void handleCopy()}
      >
        <Copy />
      </FidelityIconButton>
      <FidelityIconButton
        size="sm"
        variant="secondary"
        className="rounded-[0.625rem] bg-transparent"
        aria-label={libraryText.openInExplorer()}
        title={libraryText.openInExplorer()}
        onClick={onOpen}
      >
        <FolderOpen />
      </FidelityIconButton>
    </div>
  )
}

function ToolRow({
  tool,
  onConfigure,
}: {
  tool: ToolSummary
  onConfigure: () => void
}) {
  return (
    <div className="flex items-center gap-3.5 rounded-2xl border border-border bg-secondary px-4.5 py-3.5">
      <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-[0.625rem] bg-primary/15 text-primary [&_svg]:size-4">
        <ToolIconGlyph tool={tool} />
      </span>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-bold text-foreground">
          {tool.name}
        </p>
        <p className="truncate text-[11px] uppercase tracking-[0.05em] text-muted-foreground">
          {tool.executablePath.split(/[\\/]/).pop()}
        </p>
      </div>
      <FidelityIconButton
        size="sm"
        variant="secondary"
        className="rounded-[0.625rem] bg-transparent"
        aria-label={libraryText.launchToolLabel(tool.name)}
        title={libraryText.launchToolLabel(tool.name)}
        onClick={() => void launchTool(tool)}
      >
        <Play />
      </FidelityIconButton>
      <FidelityIconButton
        size="sm"
        variant="secondary"
        className="rounded-[0.625rem] bg-transparent"
        aria-label={libraryText.configureToolLabel(tool.name)}
        title={libraryText.configureToolLabel(tool.name)}
        onClick={onConfigure}
      >
        <Settings2 />
      </FidelityIconButton>
    </div>
  )
}

/** Registered-but-unreadable entry (C13): warning row + bare path, remove-only. */
function StubDetails({ stub }: { stub: LibraryStub }) {
  const [pending, setPending] = useState(false)

  async function handleRemove() {
    setPending(true)
    try {
      // The stub has no id, only its registered path — the path is the only handle the frontend
      // can pass for the entry-only (never destructive) removal.
      await deleteLibrary({ libraryId: stub.path, deleteFiles: false })
    } finally {
      setPending(false)
    }
  }

  return (
    <>
      <div className="px-9 py-7">
        <div className="flex items-center gap-3.5 rounded-2xl border border-destructive/25 bg-destructive/10 px-4.5 py-4">
          <span className="inline-flex size-8.5 shrink-0 items-center justify-center rounded-[0.625rem] bg-destructive/15 text-destructive">
            <TriangleAlert className="size-4" aria-hidden />
          </span>
          <div className="min-w-0 flex-1">
            <p className="text-sm font-bold text-foreground">
              {libraryText.pathUnreachable()}
            </p>
            <p className="truncate text-[13px] text-muted-foreground" title={stub.path}>
              {stub.path}
            </p>
          </div>
        </div>
        <p className="mt-3 text-xs text-muted-foreground">
          {libraryText.unreadableLibraryHint()}
        </p>
      </div>
      <div className="flex items-center justify-end border-t border-border px-9 py-5">
        <FidelityButton
          variant="ghost"
          size="sm"
          className="text-[13px] font-bold text-destructive hover:text-destructive"
          busy={pending}
          onClick={() => void handleRemove()}
        >
          {libraryText.removeEntry()}
        </FidelityButton>
      </div>
    </>
  )
}

function entryKey(entry: LibraryEntry): string {
  return isLibrarySummary(entry) ? entry.id : entry.path
}

async function pickGameRootFolder(): Promise<string | null> {
  if (!isTauri()) {
    // MOCK-FALLBACK: no native folder picker in the browser prototype — simulate a selection so
    // the create flow stays exercisable end-to-end.
    console.info('[redesign] no native folder picker, simulating a selection')
    return `C:/SPT-instances/instance-${Date.now() % 1000}`
  }
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selection = await open({
    directory: true,
    multiple: false,
    title: libraryText.selectGameRoot(),
  })
  return typeof selection === 'string' ? selection : null
}
