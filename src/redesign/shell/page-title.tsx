/*
 * Page title contract between screens and the shell header (consolidated-spec.md §12: the header
 * renders page title + subtitle only). A screen declares `<PageTitle title=… subtitle=… />`;
 * `AppHeader` reads the atom. Screens own the copy, the shell owns the rendering — no header
 * portals (§12 "No old header-portal system").
 */
import { useEffect } from 'react'
import { atom, useSetAtom } from 'jotai'

export type PageTitleState = { title: string; subtitle?: string }

export const pageTitleAtom = atom<PageTitleState>({ title: '' })

export function PageTitle({ title, subtitle }: PageTitleState) {
  const setPageTitle = useSetAtom(pageTitleAtom)
  useEffect(() => {
    setPageTitle({ title, subtitle })
  }, [title, subtitle, setPageTitle])
  return null
}
