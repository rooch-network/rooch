// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useState } from 'react'
import { AppsItem, AppItemProps } from './components/apps-item'
import {useTranslation} from 'react-i18next';

const mockApps: AppItemProps[] = [
  {
    id: 1,
    name: 'Rooch Clicker',
    description: 'Join our Click Challenge! You\'re in for 1,000 RCC!',
    profileUrl:
      'https://cdn.lxdao.io/bafkreig3psglqxqiejrcokqwcoucbv4i2nkp4rumqawok2vjvhey5ps63i.png',
    logoUrl: 'clicker-app.jpg',
    type: 'Clicker',
    url: 'https://rooch-clicker.vercel.app'
  }
]

export const AppsLayout = () => {
  const [apps] = useState<AppItemProps[]>(mockApps)
  const { t } = useTranslation()

  const renderContent = () => {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 w-full place-items-center">
        {apps.map((app) => (
          <AppsItem
            key={app.id}
            id={app.id}
            name={app.name}
            description={app.description}
            profileUrl={app.profileUrl}
            logoUrl={app.logoUrl}
            type={app.type}
            url={app.url}
          />
        ))}
      </div>
    )
  }

  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      <div className="flex items-center justify-between space-y-2">
        <span>
          <h1 className="text-3xl font-bold tracking-tight">{t('Apps.title')}</h1>
          <p className="text-muted-foreground text-wrap">
            {t('Apps.subTitle')}
          </p>
        </span>
      </div>
      {renderContent()}
    </div>
  )
}
