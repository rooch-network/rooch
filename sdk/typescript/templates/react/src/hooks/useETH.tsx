// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React
import { useContext } from 'react'

// ** Context
import { ETHContext } from '@/context/wallet'

export const useETH = () => useContext(ETHContext)
