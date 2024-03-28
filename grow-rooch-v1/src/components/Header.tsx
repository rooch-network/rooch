import { useState } from 'react'
import { Dialog } from '@headlessui/react'
import { Bars3Icon, XMarkIcon } from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

interface HeaderProps {
  name: string
  href: string
}

const navigation: HeaderProps[] = [
  { name: 'Home', href: '#' },
  { name: 'Leaderboard', href: '#' },
]

export const Header = () => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const [activeItem, setActiveItem] = useState('Home')

  // Update active item
  const handleItemClick = (itemName: string) => {
    setActiveItem(itemName)
  }

  const handleJumpToX = () => {
    window.open('https://twitter.com/home', '_blank')
  }

  return (
    <header className="inset-x-0 top-0 z-50">
      <nav
        className="flex items-center justify-between p-6 lg:px-8 mx-auto max-w-7xl px-6 pb-8"
        aria-label="Global"
      >
        <div className="flex lg:flex-1">
          <a href="#" className="-m-1.5 p-1.5 hover:opacity-75 transition-all">
            <span className="sr-only">Home</span>
            <img className="h-8 w-auto" src="rooch_black_combine.svg" alt="rooch" />
          </a>
        </div>
        <div className="flex lg:hidden">
          <button
            type="button"
            className="-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-700"
            onClick={() => setMobileMenuOpen(true)}
          >
            <span className="sr-only">Dashboard</span>
            <Bars3Icon className="h-6 w-6" aria-hidden="true" />
          </button>
        </div>
        <div className="hidden lg:flex lg:gap-x-12">
          {navigation.map((item) => (
            <a
              key={item.name}
              href={item.href}
              className={cn(
                'text-sm font-semibold leading-6 text-gray-900 hover:opacity-75 transition-all',
                activeItem === item.name ? 'shadow-underline' : '',
              )}
              onClick={() => handleItemClick(item.name)}
            >
              {item.name}
            </a>
          ))}
          <a
            href="https://twitter.com/home"
            className="text-sm font-semibold leading-6 text-gray-900 hover:opacity-75 transition-all"
            onClick={() => handleJumpToX}
          >
            <span className="flex items-center justify-center gap-1">
              Share to
              <img src="/icon-x.svg" alt="" className="w-4" />
            </span>
          </a>
        </div>
        <div className="hidden lg:flex lg:flex-1 lg:justify-end">
          <a
            href="#"
            className="text-sm font-semibold leading-6 text-gray-900 hover:opacity-75 transition-all"
          >
            Connect wallet <span aria-hidden="true">&rarr;</span>
          </a>
        </div>
      </nav>
      <Dialog as="div" className="lg:hidden" open={mobileMenuOpen} onClose={setMobileMenuOpen}>
        <div className="fixed inset-0 z-50" />
        <Dialog.Panel className="fixed inset-y-0 right-0 z-50 w-full overflow-y-auto bg-white px-6 py-6 sm:max-w-sm sm:ring-1 sm:ring-gray-900/10">
          <div className="flex items-center justify-between">
            <a href="#" className="-m-1.5 p-1.5 hover:opacity-75 transition-all">
              <span className="sr-only">Home</span>
              <img className="h-8 w-auto" src="rooch_black_combine.svg" alt="rooch" />
            </a>
            <button
              type="button"
              className="-m-2.5 rounded-md p-2.5 text-gray-700"
              onClick={() => setMobileMenuOpen(false)}
            >
              <span className="sr-only">Dashboard</span>
              <XMarkIcon className="h-6 w-6" aria-hidden="true" />
            </button>
          </div>
          <div className="mt-6 flow-root">
            <div className="-my-6 divide-y divide-gray-500/10">
              <div className="space-y-2 py-6">
                {navigation.map((item) => (
                  <a
                    key={item.name}
                    href={item.href}
                    className={cn(
                      '-mx-3 block rounded-lg px-3 py-2 text-base font-semibold leading-7 text-gray-900 transition-all',
                      activeItem === item.name
                        ? 'text-indigo-800 bg-indigo-50'
                        : 'hover:bg-gray-50',
                    )}
                    onClick={() => handleItemClick(item.name)}
                  >
                    {item.name}
                  </a>
                ))}
              </div>
              <div className="py-6">
                <a
                  href="#"
                  className="-mx-3 block rounded-lg px-3 py-2.5 text-base font-semibold leading-7 text-gray-900 hover:bg-gray-50 transition-all"
                >
                  Connect wallet
                </a>
              </div>
            </div>
          </div>
        </Dialog.Panel>
      </Dialog>
    </header>
  )
}
