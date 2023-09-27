// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React
import { useContext } from 'react'

//
import { RoochContext } from 'src/context/rooch/index'

export const useRooch = () => useContext(RoochContext)
