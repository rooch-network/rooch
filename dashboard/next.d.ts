// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ReactElement, ReactNode } from 'react'
import type { NextComponentType, NextPageContext } from 'next/dist/shared/lib/utils'

declare module 'next' {
  export declare type NextPage<P = object, IP = P> = NextComponentType<NextPageContext, IP, P> & {
    authGuard?: boolean
    guestGuard?: boolean
    setConfig?: () => void
    contentHeightFixed?: boolean
    getLayout?: (page: ReactElement) => ReactNode
  }
}
