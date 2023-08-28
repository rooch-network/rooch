// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { SettingsContext, SettingsContextValue } from 'src/@core/context/settingsContext'

export const useSettings = (): SettingsContextValue => useContext(SettingsContext)
