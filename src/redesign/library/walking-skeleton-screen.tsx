import { useAtomValue } from 'jotai'
import { activeLibraryAtom, modListAtom } from '../state/library-state'
import { PageTitle } from '../shell/page-title'
import { libraryText } from '../i18n/library-text'
import { ModTitleCard } from './mod-title-card'
import { LibraryExecutionBar } from './library-execution-bar'

/**
 * Walking-skeleton composition (consolidated-spec.md §8.2) — THROWAWAY scaffolding.
 *
 * The single vertical slice that exercises every layer once: route + shell → atoms/repository →
 * plain toggle + fire-and-track sync → library-busy → card + execution bar. 8.5 replaces this with
 * the real `library-screen.tsx`; it exists only to freeze the cross-layer contracts before the
 * horizontal build-out. Since 8.3 it mounts at `/library` behind the route adapter and reports its
 * title through the shell's PageTitle contract.
 */
export function WalkingSkeletonScreen() {
  const activeLibrary = useAtomValue(activeLibraryAtom)
  const mods = useAtomValue(modListAtom)

  return (
    <div className="flex flex-col gap-6">
      <PageTitle title={libraryText.title()} subtitle={activeLibrary?.name} />

      <LibraryExecutionBar />

      <div className="flex flex-col gap-2">
        {mods.map((mod) => (
          <ModTitleCard key={mod.id} mod={mod} />
        ))}
      </div>
    </div>
  )
}
