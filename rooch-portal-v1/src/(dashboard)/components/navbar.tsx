import { MobileSidebar } from './mobile-sidebar'
import { NavbarItem } from './navbar-item'

export const Navbar = () => {
  return (
    <div className="py-4 px-4 h-full flex items-center border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="flex items-center justify-between md:justify-end w-full">
        <MobileSidebar />
        <NavbarItem />
      </div>
    </div>
  )
}
