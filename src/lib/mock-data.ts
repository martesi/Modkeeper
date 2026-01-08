import { faker } from '@faker-js/faker'
import type { LibraryDTO, LibrarySwitch, Mod, ModManifest, ModType } from '@gen/bindings'

/**
 * Generate a mock mod manifest
 */
function generateMockManifest(): ModManifest {
  const name = faker.word.words({ count: { min: 1, max: 3 } })
  const id = name.toLowerCase().replace(/\s+/g, '-')
  const version = `${faker.number.int({ min: 1, max: 9 })}.${faker.number.int({ min: 0, max: 20 })}.${faker.number.int({ min: 0, max: 99 })}`
  const sptVersion = `3.${faker.number.int({ min: 7, max: 11 })}.${faker.number.int({ min: 0, max: 9 })}`

  const author = faker.datatype.boolean()
    ? faker.person.fullName()
    : [faker.person.fullName(), faker.person.fullName()]

  return {
    id,
    name,
    author,
    version,
    sptVersion,
    description: faker.datatype.boolean({ probability: 0.85 })
      ? faker.lorem.sentence({ min: 5, max: 20 })
      : undefined,
    icon: faker.datatype.boolean() ? 'icon.png' : undefined,
    documentation: faker.datatype.boolean() ? 'README.md' : undefined,
    compatibility: faker.datatype.boolean()
      ? {
          include: faker.datatype.boolean()
            ? [faker.word.words(2), faker.word.words(2)]
            : undefined,
          exclude: faker.datatype.boolean()
            ? [faker.word.words(2)]
            : undefined,
        }
      : undefined,
    dependencies: faker.datatype.boolean()
      ? {
          [faker.word.words(2).toLowerCase().replace(/\s+/g, '-')]: `^${faker.number.int({ min: 1, max: 5 })}.0.0`,
          [faker.word.words(2).toLowerCase().replace(/\s+/g, '-')]: `~${faker.number.int({ min: 2, max: 6 })}.5.0`,
        }
      : undefined,
    effects: faker.datatype.boolean()
      ? faker.helpers.arrayElements(['trader', 'item', 'other'] as const, { min: 1, max: 2 })
      : undefined,
    links: faker.datatype.boolean()
      ? [
          {
            type: 'website',
            name: faker.company.name(),
            url: faker.internet.url(),
          },
        ]
      : undefined,
  }
}

/**
 * Generate a mock mod type
 */
function generateModType(): ModType {
  return faker.helpers.arrayElement(['Client', 'Server', 'Both', 'Unknown'] as const)
}

/**
 * Generate a mock mod
 */
function generateMockMod(): Mod {
  const manifest = generateMockManifest()
  return {
    id: manifest.id,
    is_active: faker.datatype.boolean({ probability: 0.7 }), // 70% chance of being active
    mod_type: generateModType(),
    name: manifest.name,
    manifest,
    icon_data: faker.datatype.boolean({ probability: 0.3 })
      ? faker.image.dataUri({ width: 128, height: 128 })
      : undefined,
  }
}

/**
 * Generate a mock library
 */
export function generateMockLibrary(options?: {
  name?: string
  modCount?: number
  isDirty?: boolean
}): LibraryDTO {
  const name = options?.name ?? faker.word.words({ count: 2 })
  const id = name.toLowerCase().replace(/\s+/g, '-')
  const modCount = options?.modCount ?? faker.number.int({ min: 3, max: 15 })

  const mods: Record<string, Mod> = {}
  for (let i = 0; i < modCount; i++) {
    const mod = generateMockMod()
    mods[mod.id] = mod
  }

  return {
    id,
    name,
    game_root: faker.system.directoryPath(),
    repo_root: faker.system.directoryPath(),
    spt_version: `3.${faker.number.int({ min: 7, max: 11 })}.${faker.number.int({ min: 0, max: 9 })}`,
    mods,
    is_dirty: options?.isDirty ?? faker.datatype.boolean({ probability: 0.2 }),
  }
}

/**
 * Generate a mock library switch with multiple libraries
 */
export function generateMockLibrarySwitch(options?: {
  libraryCount?: number
  activeIndex?: number
}): LibrarySwitch {
  const libraryCount = options?.libraryCount ?? faker.number.int({ min: 1, max: 5 })
  const libraries: LibraryDTO[] = []

  for (let i = 0; i < libraryCount; i++) {
    libraries.push(generateMockLibrary({
      name: `${faker.word.words(2)} Library`,
      modCount: faker.number.int({ min: 2, max: 12 }),
    }))
  }

  const activeIndex = options?.activeIndex ?? 0
  const active = libraries[activeIndex] ?? null

  return {
    active,
    libraries,
  }
}

/**
 * Create a singleton instance for consistent mock data across the app
 */
class MockDataStore {
  private librarySwitch: LibrarySwitch | null = null

  initialize() {
    if (!this.librarySwitch) {
      this.librarySwitch = generateMockLibrarySwitch({
        libraryCount: 3,
        activeIndex: 0,
      })
    }
  }

  getLibrarySwitch(): LibrarySwitch | null {
    this.initialize()
    return this.librarySwitch
  }

  getActiveLibrary(): LibraryDTO | null {
    this.initialize()
    return this.librarySwitch?.active ?? null
  }

  setActiveLibrary(libraryId: string): LibrarySwitch | null {
    this.initialize()
    if (!this.librarySwitch) return null

    const library = this.librarySwitch.libraries.find(lib => lib.id === libraryId)
    if (!library) return null

    this.librarySwitch = {
      ...this.librarySwitch,
      active: library,
    }
    return this.librarySwitch
  }

  addLibrary(library: LibraryDTO): LibrarySwitch {
    this.initialize()
    if (!this.librarySwitch) {
      this.librarySwitch = {
        active: library,
        libraries: [library],
      }
    } else {
      this.librarySwitch = {
        active: library,
        libraries: [...this.librarySwitch.libraries, library],
      }
    }
    return this.librarySwitch
  }

  updateActiveLibrary(updater: (library: LibraryDTO) => LibraryDTO): LibraryDTO | null {
    this.initialize()
    if (!this.librarySwitch?.active) return null

    const updated = updater(this.librarySwitch.active)
    this.librarySwitch = {
      ...this.librarySwitch,
      active: updated,
      libraries: this.librarySwitch.libraries.map(lib =>
        lib.id === updated.id ? updated : lib
      ),
    }
    return updated
  }
}

export const mockDataStore = new MockDataStore()
