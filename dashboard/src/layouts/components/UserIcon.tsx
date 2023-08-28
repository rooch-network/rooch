// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IconProps } from '@iconify/react'

// ** Custom Icon Import
import Icon from 'src/@core/components/icon'

const UserIcon = ({ icon, ...rest }: IconProps) => {
  return <Icon icon={icon} {...rest} />
}

export default UserIcon
