import { createFileRoute } from '@tanstack/react-router'
import { t, msg } from '@lingui/core/macro'

export const Route = createFileRoute('/library')({
  component: RouteComponent,
  staticData: {
    breadcrumb: () => t(msg`Library`),
  },
})

function RouteComponent() {
  return <Outlet />
}
