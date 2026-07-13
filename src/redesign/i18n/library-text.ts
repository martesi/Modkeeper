/*
 * Library-screen i18n namespace (frontend-redesign-spec.md §10): screen, toolbar, empty states,
 * bulk actions, and the Manage Library dialog.
 */
import { t } from '@lingui/core/macro'

export const libraryText = {
  title: () => t({ id: 'library.header.title', message: 'Library' }),
  subtitleEmpty: () =>
    t({
      id: 'library.header.subtitleEmpty',
      message: 'Click to create or activate a library',
    }),
  sync: () => t({ id: 'library.actions.sync', message: 'Sync' }),
  syncing: () => t({ id: 'library.status.syncing', message: 'Syncing…' }),
  deployStale: () =>
    t({ id: 'library.status.deployStale', message: 'Deployment out of date' }),
  deployUpToDate: () =>
    t({
      id: 'library.status.deployUpToDate',
      message: 'Deployment up to date',
    }),
  enableMod: (name: string) =>
    t({ id: 'library.card.enableMod', message: `Enable ${name}` }),
  selectMod: (name: string) =>
    t({ id: 'library.card.selectMod', message: `Select ${name}` }),
  taskItemsFailed: (count: number) =>
    t({
      id: 'library.status.taskItemsFailed',
      message: `${count} item(s) failed`,
    }),

  // Import surfaces
  addMods: () => t({ id: 'library.actions.addMods', message: 'Add mods' }),
  dropZipPrompt: () =>
    t({
      id: 'library.empty.dropZipPrompt',
      message: 'Drop .zip archives here, or click to browse',
    }),
  manageLibraries: () =>
    t({ id: 'library.actions.manageLibraries', message: 'Manage libraries' }),
  zipArchive: () =>
    t({ id: 'library.import.zipArchive', message: 'Zip archive' }),
  selectModArchives: () =>
    t({
      id: 'library.import.selectModArchives',
      message: 'Select mod archives',
    }),
  nonZipRejected: () =>
    t({
      id: 'library.import.nonZipRejected',
      message: 'Only .zip archives can be imported',
    }),

  // Toolbar
  selectAllVisible: () =>
    t({
      id: 'library.toolbar.selectAllVisible',
      message: 'Select all visible mods',
    }),
  searchPlaceholder: () =>
    t({ id: 'library.toolbar.searchPlaceholder', message: 'Search mods' }),
  searchLabel: () =>
    t({ id: 'library.toolbar.searchLabel', message: 'Search mods' }),
  sortNameAsc: () =>
    t({ id: 'library.toolbar.sortNameAsc', message: 'Sorted A to Z' }),
  sortNameDesc: () =>
    t({ id: 'library.toolbar.sortNameDesc', message: 'Sorted Z to A' }),
  filterLabel: () =>
    t({ id: 'library.toolbar.filterLabel', message: 'Filter by type' }),
  filterAll: () => t({ id: 'library.toolbar.filterAll', message: 'All types' }),
  typeClient: () => t({ id: 'library.modType.client', message: 'Client' }),
  typeServer: () => t({ id: 'library.modType.server', message: 'Server' }),
  typeBoth: () => t({ id: 'library.modType.both', message: 'Both' }),
  typeUnknown: () => t({ id: 'library.modType.unknown', message: 'Unknown' }),
  actions: (count: number) =>
    t({ id: 'library.toolbar.actions', message: `Actions (${count})` }),
  noVisibleMods: () =>
    t({
      id: 'library.grid.noVisibleMods',
      message: 'No mods match the current filter',
    }),

  // Bulk actions
  enableSelected: () =>
    t({ id: 'library.bulk.enableSelected', message: 'Enable selected' }),
  disableSelected: () =>
    t({ id: 'library.bulk.disableSelected', message: 'Disable selected' }),
  deleteSelected: () =>
    t({ id: 'library.bulk.deleteSelected', message: 'Delete selected' }),
  deleteModsTitle: (count: number) =>
    t({
      id: 'library.bulk.deleteModsTitle',
      message: `Delete ${count} mod(s)?`,
    }),
  deleteModsDescription: () =>
    t({
      id: 'library.bulk.deleteModsDescription',
      message: 'The selected mods are removed from the library on disk.',
    }),
  deleteConfirm: () =>
    t({ id: 'library.bulk.deleteConfirm', message: 'Delete' }),

  // Manage Library dialog
  manageDialogTitle: () =>
    t({ id: 'library.manage.dialogTitle', message: 'Manage Libraries' }),
  addLibraryTab: () =>
    t({ id: 'library.manage.addLibraryTab', message: 'Add library' }),
  selectGameRoot: () =>
    t({
      id: 'library.manage.selectGameRoot',
      message: 'Select the SPT game folder',
    }),
  identitySection: () =>
    t({ id: 'library.manage.identitySection', message: 'Identity' }),
  nameLabel: () => t({ id: 'library.manage.nameLabel', message: 'Name' }),
  saveName: () => t({ id: 'library.manage.saveName', message: 'Save' }),
  sptVersion: (version: string) =>
    t({ id: 'library.manage.sptVersion', message: `SPT ${version}` }),
  pathsSection: () =>
    t({ id: 'library.manage.pathsSection', message: 'Paths' }),
  gameRootLabel: () =>
    t({ id: 'library.manage.gameRootLabel', message: 'Game root' }),
  libraryRootLabel: () =>
    t({ id: 'library.manage.libraryRootLabel', message: 'Library root' }),
  copyPath: () => t({ id: 'library.manage.copyPath', message: 'Copy path' }),
  pathCopied: () =>
    t({ id: 'library.manage.pathCopied', message: 'Path copied' }),
  openInExplorer: () =>
    t({
      id: 'library.manage.openInExplorer',
      message: 'Open in file explorer',
    }),
  rebuildCache: () =>
    t({ id: 'library.manage.rebuildCache', message: 'Rebuild cache' }),
  rebuildNeedsActive: () =>
    t({
      id: 'library.manage.rebuildNeedsActive',
      message: 'Activate this library to rebuild its cache',
    }),
  activate: () => t({ id: 'library.manage.activate', message: 'Activate' }),
  activated: () => t({ id: 'library.manage.activated', message: 'Activated' }),
  deleteLibrary: () =>
    t({ id: 'library.manage.deleteLibrary', message: 'Delete' }),
  deleteLibraryTitle: (name: string) =>
    t({ id: 'library.manage.deleteLibraryTitle', message: `Delete ${name}?` }),
  deleteLibraryDescription: () =>
    t({
      id: 'library.manage.deleteLibraryDescription',
      message: 'The library entry is removed from Modkeeper.',
    }),
  deleteFilesLabel: () =>
    t({
      id: 'library.manage.deleteFilesLabel',
      message: 'Also delete the library files on disk',
    }),
  unreadableLibrary: () =>
    t({
      id: 'library.manage.unreadableLibrary',
      message: 'Unreadable library',
    }),
  unreadableLibraryHint: () =>
    t({
      id: 'library.manage.unreadableLibraryHint',
      message:
        'This registered library could not be read. It can only be removed from the list.',
    }),
  removeEntry: () =>
    t({ id: 'library.manage.removeEntry', message: 'Remove entry' }),
}
