import { msg, t } from '@lingui/core/macro'

// avoid translation before i18n is loaded
export const DIVIDER = () => t(msg`, `)

export const getUnknownModName = () => t(msg`Unknown mod`)
