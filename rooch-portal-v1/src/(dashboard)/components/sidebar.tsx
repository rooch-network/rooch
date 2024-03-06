import { ProfileInfo } from '@/components/profile-info'
import { Logo } from './logo'
import { SidebarRoutes } from './sidebar-routes'

interface SidebarProps {
  onClose: () => void
}

export const Sidebar = ({ onClose }: SidebarProps) => {
  return (
    <div className="h-full flex flex-col overflow-y-auto dark:bg-inherit border-r shadow-sm border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="h-[85px] flex items-center justify-start px-6 py-12">
        <Logo />
      </div>
      <div className="flex flex-col w-full px-4">
        <SidebarRoutes onClose={onClose} />
      </div>
      <div className="flex w-full mt-auto p-4">
        <ProfileInfo />
      </div>
    </div>
  )
}
