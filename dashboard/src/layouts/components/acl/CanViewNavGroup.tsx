// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { ReactNode } from 'react'

// ** Hooks Imports
import { useAuth } from 'src/hooks/useAuth'

// ** Types
import { NavGroup } from 'src/@core/layouts/types'

interface Props {
  navGroup?: NavGroup
  children: ReactNode
}

const CanViewNavGroup = (props: Props) => {
  // ** Props
  const { children, navGroup } = props

  // ** Hook
  const auth = useAuth()

  if (auth.accounts || (navGroup && navGroup.auth === false)) {
    return <>{children}</>
  } else {
    return null
  }
}

export default CanViewNavGroup
