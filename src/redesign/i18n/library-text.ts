/*
 * Library-screen i18n namespace (frontend-redesign-spec.md §10). Populated as screens consume
 * members; the Composition stage (8.5) adds the toolbar/dialog/empty-state copy.
 */
import { t } from '@lingui/core/macro'

export const libraryText = {
  title: () => t({ id: 'library.header.title', message: 'Library' }),
  sync: () => t({ id: 'library.actions.sync', message: 'Sync' }),
  syncing: () => t({ id: 'library.status.syncing', message: 'Syncing…' }),
  deployStale: () =>
    t({ id: 'library.status.deployStale', message: 'Deployment out of date' }),
  deployUpToDate: () =>
    t({ id: 'library.status.deployUpToDate', message: 'Deployment up to date' }),
  enableMod: (name: string) =>
    t({ id: 'library.card.enableMod', message: `Enable ${name}` }),
  taskItemsFailed: (count: number) =>
    t({ id: 'library.status.taskItemsFailed', message: `${count} item(s) failed` }),
}
