// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useContext } from 'react'
import { MetamaskContext } from 'src/context/wallet/MetamaskContext'

export const useMetamask = () => useContext(MetamaskContext)
