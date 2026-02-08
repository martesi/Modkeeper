import { Compatibility, ModManifest } from '@gen/bindings'
import {
  AdapterEnv,
  GetModOptions,
  ModManifestWithAction,
  PaginationResponce,
} from '@typings/adpter'
import {
  FikaCompatibility,
  ForgeMod,
  ForgeModVersion,
  ForgePaginatedResponse,
  ForgeResponse,
} from '@typings/forge'
import ky, { KyInstance } from 'ky'
import { lt } from 'semver'
import { prop } from 'remeda'

interface ForgeAdapterConfig {
  baseUrl: string
  token: string
}
export class ForgeAdapter {
  client!: KyInstance
  constructor(
    config: ForgeAdapterConfig,
    private readonly env: AdapterEnv,
  ) {
    this.update(config)
  }

  update(config: ForgeAdapterConfig) {
    this.client = ky.create({
      prefixUrl: `${config.baseUrl}/api/v0/`,
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
        Authorization: `Bearer ${config.token}`,
      },
    })
  }

  async getMods(
    options: GetModOptions,
  ): Promise<PaginationResponce<ModManifest>> {
    return this.client
      .get<ForgePaginatedResponse<ForgeMod[]>>('mods', {
        searchParams: {
          page: options.page,
          per_page: options.pageSize,
          query: options.search,
        },
      })
      .json()
      .then((r) => ({
        page: r.meta.current_page,
        pageSize: r.meta.per_page,
        total: r.meta.total,
        data: r.data.map(toSimpleModManifestWithAction(this.client, this.env)),
      }))
  }
}

function toSimpleModManifestWithAction(
  client: KyInstance,
  env: AdapterEnv,
): (mod: ForgeMod) => ModManifestWithAction {
  return (mod: ForgeMod) => ({
    id: mod.guid,
    name: mod.name,
    version: '',
    sptVersion: '',
    author: [mod.owner.name, ...mod.additional_authors.map(prop('name'))],
    icon: mod.thumbnail,
    toDetail: createToDetailHandler(client, env, mod.id),
  })
}

function createToDetailHandler(
  client: KyInstance,
  env: AdapterEnv,
  id: number,
) {
  return async (): Promise<ModManifestWithAction> => {
    const reqDetail = client
      .get<ForgeResponse<ForgeMod>>(`mod/${id}`)
      .json()
      .then(prop('data'))

    const reqVersions = client
      .get<ForgePaginatedResponse<ForgeModVersion[]>>(`mod/${id}/versions`, {
        searchParams: {
          page: 1,
          per_page: 1,
          'filter[spt_version]': env.sptVersion,
          include: 'dependencies',
        },
      })
      .json()
      .then(prop('data'))
      .then((v) => v[0] as ForgeModVersion | undefined)

    const [d, v] = await Promise.all([reqDetail, reqVersions])

    return {
      ...toSimpleModManifestWithAction(client, env)(d),
      version: v?.version || '',
      sptVersion: v?.spt_version_constraint || '',
      documentation: d.description,
      compatibility: fikaToCompatibility(d.fika_compatibility),
      dependencies:
        v?.dependencies.map((dep) => {
          const version = (dep.versions || [])
            .map((v) => v.version)
            .sort((a, b) => (lt(a, b) ? -1 : 1))[0]
          return {
            id: dep.guid,
            version: version ? `^${version}` : '*',
          }
        }) || [],
    }
  }
}

function fikaToCompatibility(
  fika: FikaCompatibility,
): Compatibility | undefined {
  switch (fika) {
    case 'compatible':
      return {
        include: ['Fika', 'com.fika.core'],
      }
    case 'incompatible':
      return {
        exclude: ['Fika', 'com.fika.core'],
      }
    case 'unknown':
      return void 0
  }
}
