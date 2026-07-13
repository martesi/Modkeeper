import { createFileRoute, Outlet } from '@tanstack/react-router'

/** Outlet-only adapter (consolidated-spec.md §10 route table). */
export const Route = createFileRoute('/library')({
  component: Outlet,
})
