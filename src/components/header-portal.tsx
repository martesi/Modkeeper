import { HeaderPortalContext as Context } from '@/utils/header-portal-context'
import { PropsWithChildren, useContext } from 'react'
import { createPortal } from 'react-dom'

export function HeaderPortal ({ children }: PropsWithChildren) {
  const container = useContext(Context)
  if (!container) return null
  return createPortal(children, container)
}
