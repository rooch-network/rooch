// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { ReactNode } from 'react'

// ** Types
import { NavLink } from 'src/@core/layouts/types'

interface Props {
  navLink?: NavLink
  children: ReactNode
}

const CanViewNavLink = (props: Props) => {
  // ** Props
  const { children, navLink } = props

  if (navLink?.domain) {
    if (navLink.domain === window.location.host) {
      return <>{children}</>
    }

    return <></>
  }

  return <>{children}</>
}

export default CanViewNavLink
