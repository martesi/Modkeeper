import { Wrench } from 'lucide-react'
import type { ToolSummary } from '../data/redesign-types'

/** A tool's icon: its configured icon URL when set, a wrench glyph otherwise. */
export function ToolIconGlyph({ tool }: { tool: ToolSummary }) {
  if (tool.iconDataUrl) {
    return (
      <img
        src={tool.iconDataUrl}
        alt=""
        className="size-4 rounded-sm object-cover"
      />
    )
  }
  return <Wrench aria-hidden />
}
