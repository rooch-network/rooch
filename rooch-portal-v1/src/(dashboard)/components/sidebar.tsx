import { ProfileInfo } from '@/components/profile-info'
import { Logo } from './logo'
import { SidebarRoutes } from './sidebar-routes'
import { ModeToggle } from '@/components/mode-toggle'
import { LanguageSwitcher } from '@/components/language-switcher'
import { Separator } from '@/components/ui/separator'

interface SidebarProps {
  onClose: () => void
}

export const Sidebar = ({ onClose }: SidebarProps) => {
  return (
    <div className="h-full flex flex-col overflow-y-auto dark:bg-inherit border-r bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="h-[85px] flex items-center justify-start px-4 py-12">
        <Logo />
      </div>
      <div className="flex flex-col w-full px-4">
        <SidebarRoutes onClose={onClose} />
      </div>
      <div className="flex flex-col w-full mt-auto p-4">
        <div className="flex flex-col items-start justify-center gap-1">
          <LanguageSwitcher />
          <ModeToggle />
        </div>
        <Separator orientation="horizontal" className="m-1" />
        <ProfileInfo />
      </div>
    </div>
  )
}
