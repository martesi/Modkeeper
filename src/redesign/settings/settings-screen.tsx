import { PageTitle } from '../shell/page-title'
import { settingsText } from '../i18n/settings-text'

/**
 * Settings screen — Global-stage placeholder so `/settings` routes into the new shell (8.3 exit:
 * header, nav, routing functional). The §12.6 row list (appearance, accent, language, legacy-UI
 * switch) lands in the Composition stage (8.5).
 */
export function SettingsScreen() {
  return (
    <PageTitle title={settingsText.title()} subtitle={settingsText.subtitle()} />
  )
}
