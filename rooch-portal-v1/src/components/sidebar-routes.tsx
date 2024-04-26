import { useTranslation } from 'react-i18next'
import { SidebarItem } from './sidebar-item'
import { SidebarRoutesProps } from '@/common/interface'

import { navItems } from '@/navigation'

export const SidebarRoutes = ({ onClose }: SidebarRoutesProps) => {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col w-full space-y-1">
      {navItems().map((item) => (
        <SidebarItem
          key={item.path}
          icon={item.icon}
          label={t(item.label)}
          href={item.path}
          onClose={onClose}
        />
      ))}
    </div>
  )
}
