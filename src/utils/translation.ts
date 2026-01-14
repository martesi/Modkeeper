import { msg, t } from '@lingui/core/macro'

// avoid translation before i18n is loaded
export const tDivider = () => t(msg`, `)

export const tUnknownModName = () => t(msg`Unknown mod`)
