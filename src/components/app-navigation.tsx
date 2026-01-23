import { Link, useMatches } from '@tanstack/react-router'
import { Home, Settings } from 'lucide-react'
import { Button } from './ui/button'

export function AppNavigation() {
  const matches = useMatches()
  const currentPath = matches[matches.length - 1]?.pathname ?? '/'
  const isLibraryActive = matches.some((match) =>
    match.pathname.startsWith('/library'),
  )

  return (
    <>
      <div className="h-20" />
      <div className="fixed bottom-6 left-0 right-0 flex justify-center pointer-events-none">
        <div className="flex items-center gap-2 p-2 bg-background/80 backdrop-blur-md border rounded-full shadow-lg pointer-events-auto">
          <Button
            variant={isLibraryActive ? 'secondary' : 'ghost'}
            size="icon"
            asChild
            className="rounded-full"
          >
            <Link to="/library">
              <Home className="size-5" />
            </Link>
          </Button>
          <Button
            variant={currentPath === '/settings' ? 'secondary' : 'ghost'}
            size="icon"
            asChild
            className="rounded-full"
          >
            <Link to="/settings">
              <Settings className="size-5" />
            </Link>
          </Button>
        </div>
      </div>
    </>
  )
}
