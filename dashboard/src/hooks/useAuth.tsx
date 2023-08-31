// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { AuthContext } from 'src/context/auth/AuthContext'

export const useAuth = () => useContext(AuthContext)
