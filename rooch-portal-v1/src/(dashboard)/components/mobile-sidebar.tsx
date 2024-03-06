import { useState } from 'react'
import { Menu } from 'lucide-react'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { Sidebar } from './sidebar'

export const MobileSidebar = () => {
  const [isOpen, setIsOpen] = useState(false)

  const closeSheet = () => setIsOpen(false)

  return (
    <Sheet open={isOpen} onOpenChange={setIsOpen}>
      <SheetTrigger className="md:hidden hover:opacity-75 transition" asChild>
        <button onClick={() => setIsOpen(true)}>
          <Menu />
        </button>
      </SheetTrigger>
      <SheetContent side="left" className="p-0">
        <Sidebar onClose={closeSheet} />
      </SheetContent>
    </Sheet>
  )
}
