// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { AuthContext } from 'src/context/AuthContext'

export const useAuth = () => useContext(AuthContext)
