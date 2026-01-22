import { msg, t } from '@lingui/core/macro'

// avoid translation before i18n is loaded
export const tDivider = () => t(msg`, `)
export const tUnknownModName = () => t(msg`Unknown mod`)
export const tSelectModFiles = () => t(msg`Select Mod Files`)
export const tSelectModFolder = () => t(msg`Select Mod Folder`)
export const tSelectGameRootDirectory = () => t(msg`Select Game Root Directory`)
export const tArchive = () => t(msg`Archive`)