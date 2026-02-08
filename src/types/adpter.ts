import { ModManifest } from 'src/gen/bindings'

export interface GetModOptions {
  page?: number
  pageSize?: number
  search?: string
}

export interface PaginationResponce<T> {
  page: number
  pageSize: number
  total: number
  data: T[]
}

export interface ModManifestWithAction extends ModManifest {
  toDetail?: () => Promise<ModManifestWithAction>
  download?: () => Promise<void>
  visit?: () => Promise<void>
}

export interface AdapterEnv {
  sptVersion: string
}
