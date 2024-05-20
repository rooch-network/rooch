// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useEffect, useState } from 'react'
import { useTheme } from '@/components/theme-provider'
import { useNavigate } from 'react-router-dom'

export const Logo = () => {
  const { theme } = useTheme()
  const navigate = useNavigate()
  const [logoSrc, setLogoSrc] = useState<string>('/rooch_black_combine.svg')

  const onClick = () => {
    navigate('/')
  }

  useEffect(() => {
    switch (theme) {
      case 'dark': {
        setLogoSrc('/rooch_white_combine.svg')
        break
      }
      case 'light': {
        setLogoSrc('/rooch_black_combine.svg')
        break
      }
      case 'system':
      default: {
        // Check the system theme
        const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        setLogoSrc(systemTheme === 'dark' ? '/rooch_white_combine.svg' : '/rooch_black_combine.svg')
        break
      }
    }
  }, [theme])

  return (
    <img
      src={logoSrc}
      alt="Logo"
      onClick={onClick}
      className="cursor-pointer hover:opacity-75 transition md:h-[70px] md:w-[145px] h-[70px] w-[120px]"
    />
  )
}
