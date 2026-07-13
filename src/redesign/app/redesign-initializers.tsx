import { useEffect } from 'react'
import { toast } from 'sonner'
import {
  initWorkspaceEventBus,
  listenWorkspaceEvent,
  loadLibraryWorkspace,
} from '../data/library-repository'
import { setWorkspace } from '../state/library-state'
import { libraryText } from '../i18n/library-text'
import type { WorkspaceEvent } from '../data/redesign-types'

/**
 * App-wide startup wiring (consolidated-spec.md §12 "Initializers").
 *
 * Registers the ONE persistent workspace-event subscriber before any fire-and-track op can fire,
 * connects the real Tauri event source, and loads the initial workspace (real + mock fallback).
 * The subscriber is the single writer that applies completion events to the workspace atom. A
 * startup config warning rides the first workspace and is toasted once (C12).
 *
 * Still to come per §12: settings restore + locale init (8.4, with the settings repository) and
 * the global .zip drag/drop listener (8.5, with the drop empty states that consume it).
 */
export function RedesignInitializers() {
  useEffect(() => {
    const unsubscribe = listenWorkspaceEvent(handleWorkspaceEvent)
    void initWorkspaceEventBus()
    void loadLibraryWorkspace().then((workspace) => {
      if (workspace.configWarning) toast.warning(workspace.configWarning)
    })
    return unsubscribe
  }, [])

  return null
}

function handleWorkspaceEvent(event: WorkspaceEvent): void {
  setWorkspace(event.workspace)
  if (event.type !== 'cache_rebuild_completed' && event.failures.length > 0) {
    toast.error(libraryText.taskItemsFailed(event.failures.length))
  }
}
