/*
 * Import-surface file filters (consolidated-spec.md §12.2): the whole import path — picker and
 * window drop alike — accepts `.zip` only.
 */

export function isZipPath(path: string): boolean {
  return /\.zip$/i.test(path)
}

export function splitZipPaths(paths: string[]): {
  zips: string[]
  rejected: string[]
} {
  const zips: string[] = []
  const rejected: string[] = []
  for (const path of paths) (isZipPath(path) ? zips : rejected).push(path)
  return { zips, rejected }
}
