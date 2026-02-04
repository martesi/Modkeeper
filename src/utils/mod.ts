/**
 * Formats a Unix timestamp (in seconds) to a localized date string.
 * @param timestamp - Unix timestamp as a string (in seconds)
 * @returns Formatted date string or the original timestamp if parsing fails
 */
export function formatTimestamp(timestamp: string): string {
  const num = Number(timestamp)
  if (Number.isNaN(num)) return timestamp
  return new Date(num * 1000).toLocaleString()
}

/**
 * Gets a human-readable name for a link type.
 * @param type - The link type string
 * @returns A human-readable name for the link type
 */
export function getLinkTypeName(type: string | undefined | null): string {
  switch (type) {
    case 'code':
      return 'Source Code'
    case 'discord':
      return 'Discord'
    case 'website':
      return 'Website'
    case 'documentation':
      return 'Documentation'
    default:
      return 'Link'
  }
}
