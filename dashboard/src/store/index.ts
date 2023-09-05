// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Toolkit imports
import { configureStore } from '@reduxjs/toolkit'

// ** Reducers
import transaction from 'src/store/transaction'

export const store = configureStore({
  reducer: {
    transaction,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: false,
    }),
})

export type AppDispatch = typeof store.dispatch
export type RootState = ReturnType<typeof store.getState>
