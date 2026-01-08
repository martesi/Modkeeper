# Mock Data Setup

This project is currently configured to use mock data from `@faker-js/faker` for UI development.

## What's Mocked

The following functionality now uses mock data instead of calling Tauri backend commands:

- **Library data** - Mock libraries with random names, paths, and SPT versions
- **Mods** - Randomly generated mods with manifests, dependencies, effects, and icons
- **Library operations** - Create, open, fetch library operations
- **Mod operations** - Add, remove, toggle, sync mods
- **Mod details** - Documentation and backups

## Files Modified

### Core Mock Data
- `src/lib/mock-data.ts` - Contains mock data generators and a singleton store

### Updated to Use Mock Data
- `src/store/library-actions.ts` - All Jotai actions now use mock data
- `src/routes/mod.$id.tsx` - Documentation and backups use mock data

### Unchanged (still functional)
- `src/lib/api.ts` - Still exports real commands for future use
- All UI components - Work exactly the same way

## Switching Back to Real Data

To switch back to using real Tauri backend commands:

1. In `src/store/library-actions.ts`, uncomment the real imports:
   ```typescript
   import { commands } from '@/lib/api'
   import { unwrapResult } from '@/lib/result'
   ```

2. Replace each mock implementation with the real command calls (see git history for original implementations)

3. In `src/routes/mod.$id.tsx`, restore the original `useEffect` hooks for documentation and backups

4. Remove or comment out mock data imports

## Benefits of Mock Data

- **Fast UI Development** - No need to run the Tauri backend
- **Consistent Test Data** - Always have data to work with
- **Easy State Testing** - Test different scenarios (dirty libraries, various mod counts, etc.)
- **Browser Development** - Can develop in the browser without Tauri (with minor adjustments)

## Current Mock Data Features

The mock data includes:
- 3 libraries with random names
- 3-15 mods per library (randomized)
- Realistic mod manifests with:
  - Random authors, versions, descriptions
  - Dependencies (sometimes)
  - Effects, compatibility info
  - Links to external resources
  - Icon data (sometimes)
- Simulated network delays (300-1500ms) for realistic feel
- State persistence across operations

## Customizing Mock Data

You can customize the mock data generation in `src/lib/mock-data.ts`:

```typescript
// Change initial library count
generateMockLibrarySwitch({ libraryCount: 5 })

// Change mods per library
generateMockLibrary({ modCount: 20 })

// Force dirty state
generateMockLibrary({ isDirty: true })
```

## Notes

- Mock data is generated on first access and persists in memory
- Changes to mods (add, remove, toggle) update the mock store
- Console logs show when mock operations are triggered
- All mock operations include simulated delays to mimic real backend calls
