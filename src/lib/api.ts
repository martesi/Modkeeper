// Re-export commands and types for convenience
export { commands } from '@gen/bindings'
export type { Result, SError, LibraryDTO, LibrarySwitch } from '@gen/bindings'

// Note: For UI development, mock data is being used via the store/library-actions
// This file re-exports the real commands for when they're needed