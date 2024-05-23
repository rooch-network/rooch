// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useLocation, useNavigate } from 'react-router-dom'

import { Button } from '@/components/ui/button'
import { SidebarItemProps } from '@/common/interface'

import { cn } from '@/lib/utils'

export const SidebarItem = ({ icon: Icon, label, href, onClose }: SidebarItemProps) => {
  const { pathname } = useLocation()
  const navigate = useNavigate()

  const isActive =
    (pathname === '/' && href === '/') || pathname === href || pathname.startsWith(`${href}/`)

  const onClick = () => {
    navigate(href)
    if (onClose) {
      onClose()
    }
  }

  return (
    <Button
      onClick={onClick}
      type="button"
      variant="ghost"
      size="lg"
      className={cn(
        'flex items-center justify-start text-zinc-500 dark:text-muted-foreground hover:dark-white text-sm font-[500] transition-all hover:text-zinc-600 dark:hover:text-white hover:bg-zinc-300/20 dark:hover:bg-zinc-800/50 px-2',
        isActive &&
          'text-zinc-700 dark:text-white bg-zinc-200/50 dark:bg-zinc-800/90 hover:bg-zinc-200/50 dark:hover:bg-zinc-800/90 hover:text-zinc-700 dark:hover:text-white',
      )}
    >
      <div className="flex items-center gap-x-3 dark:hover:text-white">
        <Icon size={22} className={cn(isActive && 'text-zinc-700 dark:text-white')} />
        {label}
      </div>
    </Button>
  )
}
