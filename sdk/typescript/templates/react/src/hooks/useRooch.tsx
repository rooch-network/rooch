// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React
import { useContext } from 'react'

// ** Context
import { RoochContext } from '@/context/rooch'

export const useRooch = () => useContext(RoochContext)
