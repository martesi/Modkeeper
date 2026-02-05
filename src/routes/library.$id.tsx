import { createFileRoute } from '@tanstack/react-router'
import { commands } from '@gen/bindings'
import { ur } from '@/utils/result'
import { ett } from '@/utils/error'

export const Route = createFileRoute('/library/$id')({
  loader: async ({ params: { id } }) => {
    const [backups, documentation] = await Promise.all([
      commands
        .getBackups(id)
        .then(ur)
        .catch((v) => {
          ett(v)
          return []
        }),
      commands
        .getModDocumentation(id)
        .then(ur)
        .catch(() => null),
    ])

    return { backups, documentation }
  },
})
