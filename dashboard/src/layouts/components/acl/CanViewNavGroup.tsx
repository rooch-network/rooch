// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { ReactNode } from 'react'

// ** Types
import { NavGroup } from 'src/@core/layouts/types'

interface Props {
  navGroup?: NavGroup
  children: ReactNode
}

const CanViewNavGroup = (props: Props) => {
  // ** Props
  const { children, navGroup } = props

  if (navGroup?.domain) {
    if (navGroup.domain === window.location.host) {
      return <>{children}</>
    }

    return <></>
  }

  return <>{children}</>
}

export default CanViewNavGroup
