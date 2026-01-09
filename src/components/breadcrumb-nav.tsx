import { useMatches, Link } from '@tanstack/react-router'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@comps/breadcrumb.tsx'
import { useMemo, Fragment } from 'react'

export function BreadcrumbNav() {
  const matches = useMatches()

  const breadcrumbs = useMemo(() => {
    const items: Array<{ label: string; to?: string }> = []

    // Add breadcrumbs from route matches (skip root route)
    for (const match of matches.slice(1)) {
      const staticData = match.staticData as
        | { breadcrumb?: () => string }
        | undefined
      const breadcrumb = staticData?.breadcrumb
      if (breadcrumb) {
        const label =
          typeof breadcrumb === 'function' ? breadcrumb() : breadcrumb
        const to = match.pathname
        items.push({ label, to })
      }
    }

    return items
  }, [matches])

  if (breadcrumbs.length === 0) {
    return null
  }

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {breadcrumbs.map((item, index) => {
          const isLast = index === breadcrumbs.length - 1
          return (
            <Fragment key={index}>
              {index > 0 && <BreadcrumbSeparator />}
              <BreadcrumbItem>
                {isLast ? (
                  <BreadcrumbPage>{item.label}</BreadcrumbPage>
                ) : (
                  <BreadcrumbLink asChild>
                    <Link to={item.to || '/'}>{item.label}</Link>
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
            </Fragment>
          )
        })}
      </BreadcrumbList>
    </Breadcrumb>
  )
}
