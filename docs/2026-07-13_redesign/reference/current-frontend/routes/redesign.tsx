import { createFileRoute } from '@tanstack/react-router'
import { RedesignRoot } from '@/redesign/app/redesign-root'
import { WalkingSkeletonScreen } from '@/redesign/library/walking-skeleton-screen'

/**
 * Temporary dev route for the walking skeleton (consolidated-spec.md §8.2 "minimal route + shell
 * mount"). Reachable at /redesign so the slice renders in the real app, not just Storybook. 8.3's
 * route adapters + new/old UI toggle replace this entry point.
 */
export const Route = createFileRoute('/redesign')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <RedesignRoot>
      <WalkingSkeletonScreen />
    </RedesignRoot>
  )
}
