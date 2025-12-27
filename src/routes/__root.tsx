import { Outlet, createRootRoute } from '@tanstack/react-router'
import { Fragment } from 'react'

export const Route = createRootRoute({
  component: RootComponent,
})

function RootComponent() {
  return (
    <Fragment>
      <Outlet />
    </Fragment>
  )
}
