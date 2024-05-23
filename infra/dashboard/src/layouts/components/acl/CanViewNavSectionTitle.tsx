// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { ReactNode } from 'react'

// ** Types
import { NavSectionTitle } from 'src/@core/layouts/types'

interface Props {
  children: ReactNode
  navTitle?: NavSectionTitle
}

const CanViewNavSectionTitle = (props: Props) => {
  // ** Props
  const { children, navTitle } = props

  if (navTitle?.domain) {
    if (navTitle.domain === window.location.host) {
      return <>{children}</>
    }

    return <></>
  }

  return <>{children}</>
}

export default CanViewNavSectionTitle
