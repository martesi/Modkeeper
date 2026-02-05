import { createFileRoute } from '@tanstack/react-router'
import { Trans } from '@lingui/react/macro'
import { HeaderPortal } from '@/components/header-portal'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ThemeSettings } from '@/modules/settings/theme-settings'
import { LanguageSettings } from '@/modules/settings/language-settings'
import { DeveloperSettings } from '@/modules/settings/developer-settings'
import { SettingsImportExport } from '@/modules/settings/settings-import-export'
import { Separator } from '@/components/ui/separator'

export const Route = createFileRoute('/settings')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">
          <Trans>Settings</Trans>
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          <Trans>Application settings and preferences</Trans>
        </p>
      </div>

      <Tabs defaultValue="general" className="w-full">
        <HeaderPortal>
          <TabsList>
            <TabsTrigger value="general">
              <Trans>General</Trans>
            </TabsTrigger>
            <TabsTrigger value="developer">
              <Trans>Developer</Trans>
            </TabsTrigger>
          </TabsList>
        </HeaderPortal>

        <TabsContent value="general" className="space-y-6">
          <div className="border rounded-lg p-6 space-y-6">
            <ThemeSettings />
            <Separator />
            <LanguageSettings />
            <Separator />
            <SettingsImportExport />
          </div>
        </TabsContent>

        <TabsContent value="developer" className="space-y-6">
          <div className="border rounded-lg p-6">
            <DeveloperSettings />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
