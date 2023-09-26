// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { SessionContext } from 'src/context/session/SessionContext'

export const useSession = () => useContext(SessionContext)
