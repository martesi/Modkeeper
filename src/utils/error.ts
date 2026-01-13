import { toast } from 'sonner'

export function ett(error?: Error | string) {
  if (error instanceof Error) {
    toast.error(error.message)
  } else {
    toast.error(error)
  }
}
