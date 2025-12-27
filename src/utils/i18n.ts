import { i18n } from '@lingui/core'

export async function changeLocale(locale: string) {
  const { messages } = await import(`../../locales/${locale}.po`)

  i18n.loadAndActivate({ locale, messages })
}
