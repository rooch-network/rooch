// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React
import { useContext } from 'react'

//
import { ETHContext } from 'src/context/wallet/index'

export const useETH = () => useContext(ETHContext)
