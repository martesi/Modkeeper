/*
 * Library repository (consolidated-spec.md §10), walking-skeleton slice.
 *
 * The UI calls these functions, never `@gen/bindings` directly. Every function is real-first /
 * mock-fallback: it tries the generated Tauri command, and if no backend is present (browser
 * prototype) or the transport fails (backend killed), it falls back to `example-data.ts`. Every
 * fallback is tagged `MOCK-FALLBACK` so it is a single grep target that goes to zero once the
 * backend covers it.
 *
 * Two repository SHAPES are proven here and frozen by the 8.2 gate:
 *   - PLAIN (toggleModStatus): awaited, returns the updated workspace, local per-click pending only.
 *   - FIRE-AND-TRACK (syncMods): mint a client taskId, register the pending task BEFORE invoking,
 *     resolve with an accept; the OUTCOME arrives later as a WorkspaceEvent matched by taskId.
 * Both feed one single persistent event bus; the bus dispatches by taskId and clears library-busy.
 */
import { isTauri } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { commands, events } from '@gen/bindings'
import type {
  LibraryWorkspace,
  LibraryId,
  ModId,
  OperationAccepted,
  WorkspaceEvent,
  SError,
} from './redesign-types'
import { isLibrarySummary } from './redesign-types'
import { resolveCommandError } from '../shared/hooks/use-command-error'
import { getMockWorkspace, setMockWorkspace } from './example-data'
import {
  setWorkspace,
  addPendingTask,
  removePendingTask,
} from '../state/library-state'

// --- The single persistent event bus (§10) ---

type WorkspaceEventHandler = (event: WorkspaceEvent) => void

const subscribers = new Set<WorkspaceEventHandler>()
let realSourceConnected = false

/**
 * Dispatch a completion event — from the real Tauri listener OR the mock emitter, identically.
 * Clears the in-flight task (drops library-busy) first, then fans out to subscribers.
 */
function dispatchWorkspaceEvent(event: WorkspaceEvent): void {
  removePendingTask(event.taskId)
  subscribers.forEach((handler) => handler(event))
}

/** Register a workspace-event subscriber. Returns an unsubscribe. */
export function listenWorkspaceEvent(handler: WorkspaceEventHandler): () => void {
  subscribers.add(handler)
  return () => subscribers.delete(handler)
}

/**
 * Connect the real Tauri event source into the bus, once. Called by redesign-initializers. In the
 * browser prototype there is no source — the mock emitter feeds `dispatchWorkspaceEvent` directly.
 */
export async function initWorkspaceEventBus(): Promise<void> {
  if (realSourceConnected) return
  realSourceConnected = true
  if (!isTauri()) return
  try {
    await events.workspaceEvent.listen((event) =>
      dispatchWorkspaceEvent(event.payload)
    )
  } catch (error) {
    // MOCK-FALLBACK: event source unavailable — the mock emitter still drives the bus.
    console.warn('[redesign] workspace event source unavailable', error)
  }
}

function surfaceError(error: SError): void {
  toast.error(resolveCommandError(error))
  console.error('[redesign] command error', error)
}

// --- loadLibraryWorkspace (real-first / mock-fallback) ---

export async function loadLibraryWorkspace(): Promise<LibraryWorkspace> {
  try {
    if (isTauri()) {
      const result = await commands.getLibraryWorkspace()
      if (result.status === 'ok') {
        setWorkspace(result.data)
        return result.data
      }
      surfaceError(result.error)
    }
  } catch (error) {
    // MOCK-FALLBACK: backend unavailable — load the fixture instead of crashing.
    console.warn('[redesign] getLibraryWorkspace failed, using mock', error)
  }
  // MOCK-FALLBACK: no backend present (browser prototype).
  const workspace = getMockWorkspace()
  setWorkspace(workspace)
  return workspace
}

// --- toggleModStatus (PLAIN, delta 3) ---

export async function toggleModStatus(input: {
  libraryId: LibraryId
  modId: ModId
  enabled: boolean
}): Promise<LibraryWorkspace> {
  const { libraryId, modId, enabled } = input
  try {
    if (isTauri()) {
      const result = await commands.bulkUpdateMods({
        libraryId,
        modIds: [modId],
        action: enabled ? 'enable' : 'disable',
      })
      if (result.status === 'ok') {
        setWorkspace(result.data)
        return result.data
      }
      surfaceError(result.error)
      // Domain rejection: keep the current workspace unchanged.
      return getMockWorkspace()
    }
  } catch (error) {
    // MOCK-FALLBACK: backend unavailable — apply the change locally.
    console.warn('[redesign] bulkUpdateMods failed, using mock', error)
  }
  // MOCK-FALLBACK: no backend present.
  return mockToggle(libraryId, modId, enabled)
}

function mockToggle(
  libraryId: LibraryId,
  modId: ModId,
  enabled: boolean
): LibraryWorkspace {
  const workspace = structuredClone(getMockWorkspace())
  const mod = workspace.modsByLibraryId[libraryId]?.find((m) => m.id === modId)
  if (mod) {
    mod.isEnabled = enabled
    mod.updatedAt = new Date().toISOString()
  }
  // Committing mod state never deploys — it only marks the deploy stale (C3).
  const library = workspace.libraries.find(
    (lib) => isLibrarySummary(lib) && lib.id === libraryId
  )
  if (library && isLibrarySummary(library)) library.deployStale = true
  setMockWorkspace(workspace)
  setWorkspace(workspace)
  return workspace
}

// --- syncMods (FIRE-AND-TRACK, §7e) ---

export async function syncMods(
  libraryId: LibraryId
): Promise<OperationAccepted> {
  // Client-minted taskId, registered BEFORE invoking so the completion event can never race ahead
  // of a handler (§7e / C15).
  const taskId = crypto.randomUUID()
  addPendingTask({ taskId, libraryId })
  try {
    if (isTauri()) {
      const result = await commands.syncMods({ taskId, libraryId })
      if (result.status === 'ok') {
        // Accepted: the sync_completed event (matched by taskId) clears the pending task.
        return result.data
      }
      // Sync validation rejected with no event coming — clear the pending task ourselves.
      removePendingTask(taskId)
      surfaceError(result.error)
      return { accepted: false }
    }
  } catch (error) {
    // MOCK-FALLBACK: backend unavailable — simulate the accept + completion locally.
    console.warn('[redesign] syncMods failed, using mock', error)
  }
  // MOCK-FALLBACK: no backend present — accept now, emit the completion after a short delay.
  scheduleMockSyncCompletion(taskId, libraryId)
  return { accepted: true }
}

function scheduleMockSyncCompletion(taskId: string, libraryId: LibraryId): void {
  setTimeout(() => {
    const workspace = structuredClone(getMockWorkspace())
    const library = workspace.libraries.find(
      (lib) => isLibrarySummary(lib) && lib.id === libraryId
    )
    // Sync deploys and clears the stale flag.
    if (library && isLibrarySummary(library)) library.deployStale = false
    setMockWorkspace(workspace)
    dispatchWorkspaceEvent({
      type: 'sync_completed',
      taskId,
      libraryId,
      failures: [],
      workspace,
    })
  }, 700)
}
