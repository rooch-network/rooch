// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import ManageSessions from '@/pages/settings/components/manage-sessions'
import { RoochAddress } from '@/pages/settings/components/rooch-address'
import { useTranslation } from 'react-i18next'

export const SettingsLayout = () => {
  const { t } = useTranslation()
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Connected Account section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">{t('Settings.address')}</h1>
            <p className="text-muted-foreground">
              {t('Settings.addressSubTitle')}
            </p>
          </span>
        </div>
        <RoochAddress />
      </div>
      {/* Manage Sessions section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">{t('Settings.session')}</h1>
            <p className="text-muted-foreground">{t('Settings.sessionSubTitle')}</p>
          </span>
        </div>
        <ManageSessions />
      </div>
    </div>
  )
}
