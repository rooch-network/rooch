// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { ReactNode } from 'react'

// ** ateMUI Imports
import { CardProps } from '@mui/material/Card'

export type SupportLng = 'tsx' | 'jsx' | 'rust' | 'move' | 'json'

export type CodeType = {
  code: string
  lng: SupportLng
}

export type CardSnippetProps = CardProps & {
  id?: string
  title?: string
  defaultShow?: boolean
  fullHeight?: boolean
  children?: ReactNode
  codes: CodeType[]
  className?: string
}
