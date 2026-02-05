import { Link, useMatches } from '@tanstack/react-router'
import { Home, Settings } from 'lucide-react'
import { Button } from '@comps/button'

export function AppNavigation() {
  const matches = useMatches()
  const currentPath = matches[matches.length - 1]?.pathname ?? '/'
  const isLibraryActive = matches.some((match) =>
    match.pathname.startsWith('/library'),
  )

  return (
    <div className="flex items-center gap-2">
      <Button
        variant={isLibraryActive ? 'secondary' : 'ghost'}
        size="icon"
        asChild
      >
        <Link to="/library">
          <Home className="size-5" />
        </Link>
      </Button>
      <Button
        variant={currentPath === '/settings' ? 'secondary' : 'ghost'}
        size="icon"
        asChild
      >
        <Link to="/settings">
          <Settings className="size-5" />
        </Link>
      </Button>
    </div>
  )
}
