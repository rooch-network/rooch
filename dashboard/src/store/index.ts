// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Toolkit imports
import { configureStore } from '@reduxjs/toolkit'
import { TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux'

// ** Reducers
import transaction from 'src/store/scan/transaction'
import state from 'src/store/scan/state'
import session from 'src/store/session'

export const store = configureStore({
  reducer: {
    transaction,
    state,
    session,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: false,
    }),
})

export type AppDispatch = typeof store.dispatch
export type RootState = ReturnType<typeof store.getState>

export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector
export const useAppDispatch = () => useDispatch<AppDispatch>()
