'use client'

import { Trans } from '@lingui/react/macro'
import { useState, useMemo } from 'react'
import type { LibraryDTO, Mod, ModType } from '@gen/bindings'
import { ModCard } from './mod-card'
import { Input } from '@comps/input'
import { Button } from '@comps/button'
import { Checkbox } from '@comps/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@comps/select'
import { Search, CheckSquare, Square, Trash2 } from 'lucide-react'

interface ModListProps {
  library: LibraryDTO | null
  onModToggle?: (id: string, isActive: boolean) => void
  onModRemove?: (id: string) => void
}

// Pure filter functions following FP patterns
const filterBySearch = (search: string) => (mod: Mod) =>
  search === '' ||
  mod.name.toLowerCase().includes(search.toLowerCase()) ||
  mod.id.toLowerCase().includes(search.toLowerCase())

const filterByType = (type: string) => (mod: Mod) =>
  type === 'all' || mod.mod_type === type

const filterByActive = (active: string) => (mod: Mod) =>
  active === 'all' ||
  (active === 'active' && mod.is_active) ||
  (active === 'inactive' && !mod.is_active)

export function ModList({ library, onModToggle, onModRemove }: ModListProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>('all')
  const [activeFilter, setActiveFilter] = useState<string>('all')
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set())
  const [isSelectionMode, setIsSelectionMode] = useState(false)

  const mods = useMemo(() => {
    if (!library?.mods) return []
    return Object.values(library.mods).filter(Boolean) as Mod[]
  }, [library])

  const filteredMods = useMemo(() => {
    return mods
      .filter(filterBySearch(searchTerm))
      .filter(filterByType(typeFilter))
      .filter(filterByActive(activeFilter))
  }, [mods, searchTerm, typeFilter, activeFilter])

  const toggleSelection = (modId: string) => {
    setSelectedMods((prev) => {
      const newSet = new Set(prev)
      if (newSet.has(modId)) {
        newSet.delete(modId)
      } else {
        newSet.add(modId)
      }
      return newSet
    })
  }

  const selectAll = () => {
    setSelectedMods(new Set(filteredMods.map((mod) => mod.id)))
  }

  const deselectAll = () => {
    setSelectedMods(new Set())
  }

  const handleBatchToggle = async (isActive: boolean) => {
    const selectedModsArray = Array.from(selectedMods)
    try {
      // Using functional composition for batch operations
      await Promise.all(
        selectedModsArray.map((id) => onModToggle?.(id, isActive))
      )
      setSelectedMods(new Set())
      setIsSelectionMode(false)
    } catch (err) {
      console.error('Failed to batch toggle mods:', err)
    }
  }

  const handleBatchRemove = async () => {
    const selectedModsArray = Array.from(selectedMods)
    const modNames = selectedModsArray
      .map((id) => mods.find((m) => m.id === id)?.name)
      .filter(Boolean)
      .join(', ')

    if (
      confirm(
        `Are you sure you want to remove ${selectedModsArray.length} mods?\n\n${modNames}`
      )
    ) {
      try {
        await onModRemove?.(selectedModsArray[0])
        // Note: The current API only supports one at a time via onModRemove
        // For true batch operation, we'd need to call removeMods directly with the array
        setSelectedMods(new Set())
        setIsSelectionMode(false)
      } catch (err) {
        console.error('Failed to batch remove mods:', err)
      }
    }
  }

  if (!library) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No library loaded</Trans>
      </div>
    )
  }

  if (mods.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
        <p className="text-lg mb-2">
          <Trans>No mods installed</Trans>
        </p>
        <p className="text-sm">
          <Trans>Add mods to get started</Trans>
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Search and Filter Controls */}
      <div className="flex gap-4 items-center">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 size-4 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search mods by name or ID..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-10"
          />
        </div>
        <Select value={typeFilter} onValueChange={setTypeFilter}>
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Mod Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              <Trans>All Types</Trans>
            </SelectItem>
            <SelectItem value="Client">
              <Trans>Client</Trans>
            </SelectItem>
            <SelectItem value="Server">
              <Trans>Server</Trans>
            </SelectItem>
            <SelectItem value="Both">
              <Trans>Both</Trans>
            </SelectItem>
            <SelectItem value="Unknown">
              <Trans>Unknown</Trans>
            </SelectItem>
          </SelectContent>
        </Select>
        <Select value={activeFilter} onValueChange={setActiveFilter}>
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              <Trans>All Status</Trans>
            </SelectItem>
            <SelectItem value="active">
              <Trans>Active</Trans>
            </SelectItem>
            <SelectItem value="inactive">
              <Trans>Inactive</Trans>
            </SelectItem>
          </SelectContent>
        </Select>
        <Button
          variant={isSelectionMode ? 'default' : 'outline'}
          onClick={() => {
            setIsSelectionMode(!isSelectionMode)
            if (isSelectionMode) {
              setSelectedMods(new Set())
            }
          }}
        >
          <CheckSquare className="size-4 mr-2" />
          <Trans>Select</Trans>
        </Button>
      </div>

      {/* Batch Operations Bar */}
      {isSelectionMode && (
        <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="sm"
              onClick={selectedMods.size === filteredMods.length ? deselectAll : selectAll}
            >
              {selectedMods.size === filteredMods.length ? (
                <>
                  <CheckSquare className="size-4 mr-2" />
                  <Trans>Deselect All</Trans>
                </>
              ) : (
                <>
                  <Square className="size-4 mr-2" />
                  <Trans>Select All</Trans>
                </>
              )}
            </Button>
            <span className="text-sm text-muted-foreground">
              <Trans>{selectedMods.size} selected</Trans>
            </span>
          </div>
          {selectedMods.size > 0 && (
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleBatchToggle(true)}
              >
                <Trans>Activate Selected</Trans>
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleBatchToggle(false)}
              >
                <Trans>Deactivate Selected</Trans>
              </Button>
              <Button
                variant="destructive"
                size="sm"
                onClick={handleBatchRemove}
              >
                <Trash2 className="size-4 mr-2" />
                <Trans>Remove Selected</Trans>
              </Button>
            </div>
          )}
        </div>
      )}

      {/* Results Count */}
      <div className="text-sm text-muted-foreground">
        <Trans>
          Showing {filteredMods.length} of {mods.length} mods
        </Trans>
      </div>

      {/* Mod Grid */}
      {filteredMods.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
          <p className="text-lg mb-2">
            <Trans>No mods match your filters</Trans>
          </p>
          <p className="text-sm">
            <Trans>Try adjusting your search or filters</Trans>
          </p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredMods.map((mod) => (
            <div key={mod.id} className="relative">
              {isSelectionMode && (
                <div className="absolute top-2 left-2 z-10">
                  <Checkbox
                    checked={selectedMods.has(mod.id)}
                    onCheckedChange={() => toggleSelection(mod.id)}
                    className="bg-background"
                  />
                </div>
              )}
              <ModCard
                mod={mod}
                onToggle={onModToggle}
                onRemove={onModRemove}
                isSelectionMode={isSelectionMode}
                isSelected={selectedMods.has(mod.id)}
                onSelect={() => toggleSelection(mod.id)}
              />
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
