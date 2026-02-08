export interface ForgeResponse<T> {
  success: boolean
  data: T
}

export interface ForgePaginatedResponse<T> extends ForgeResponse<T> {
  links: {
    first: string
    last: string
    prev: string | null
    next: string | null
  }
  meta: {
    current_page: number
    from: number
    last_page: number
    links: {
      url: string | null
      label: string
      active: boolean
    }[]
    path: string
    per_page: number
    to: number
    total: number
  }
}

export interface ForgeModOwner {
  id: number
  name: string
  profile_photo_url: string
  cover_photo_url: string
}

export interface ForgeModCategory {
  id: number
  name: string
  slug: string
  color_class: string
}

export interface ForgeModLicense {
  id: number
  name: string
  short_name: string
}

export interface ForgeModSourceLink {
  url: string
  label: string | null
}

export type FikaCompatibility = 'compatible' | 'incompatible' | 'unknown'

export interface ForgeModVersion {
  id: number
  hub_id: number
  version: string
  description: string
  link: string
  content_length: number
  spt_version_constraint: string
  downloads: number
  fika_compatibility: FikaCompatibility
  published_at: string
  created_at: string
  updated_at: string
  dependencies: ForgeMod[]
}

export interface ForgeMod {
  id: number
  hub_id: number
  guid: string
  name: string
  slug: string
  teaser: string
  description?: string // Present in detail view, maybe optional in list
  thumbnail: string
  downloads: number
  owner: ForgeModOwner
  additional_authors: ForgeModOwner[]
  source_code_links: ForgeModSourceLink[]
  detail_url: string
  fika_compatibility: FikaCompatibility
  featured: boolean
  contains_ads: boolean
  contains_ai_content: boolean
  shows_profile_binding_notice: boolean
  published_at: string
  created_at: string
  updated_at: string

  // Included relationships
  category?: ForgeModCategory
  versions?: ForgeModVersion[]
  license?: ForgeModLicense
}
