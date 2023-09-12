// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  createSlice,
  PayloadAction,
  SliceCaseReducers,
  ValidateSliceCaseReducers,
} from '@reduxjs/toolkit'

export interface GenericState<T> {
  data: T
  error?: string | null
  status?: 'loading' | 'finished' | 'error'
}

// TODO: experimental
export const CreateGenericSlice = <T, Reducers extends SliceCaseReducers<GenericState<T>>>({
  name = '',
  initialState,
  reducers,
}: {
  name: string
  initialState: GenericState<T>
  reducers: ValidateSliceCaseReducers<GenericState<T>, Reducers>

  // extraReducers: ((builder: ActionReducerMapBuilder<NoInfer<GenericState<T>>>) => void)
}) => {
  if (initialState.status === undefined) {
    initialState.status = 'loading'
  }

  return createSlice({
    name,
    initialState,
    reducers: {
      start(state) {
        state.status = 'loading'
        state.error = null
      },

      error(state, action: PayloadAction<string | null>) {
        state.error = action.payload
        state.status = 'error'
      },

      /**
       * If you want to write to values of the state that depend on the generic
       * (in this case: `state.data`, which is T), you might need to specify the
       * State type manually here, as it defaults to `Draft<GenericState<T>>`,
       * which can sometimes be problematic with yet-unresolved generics.
       * This is a general problem when working with immer's Draft type and generics.
       */
      success(state: GenericState<T>, action: PayloadAction<T>) {
        state.data = action.payload
        state.status = 'finished'
      },
      ...reducers,
    },

    // extraReducers:{
    //   ...extraReducers
    // }
  })
}
