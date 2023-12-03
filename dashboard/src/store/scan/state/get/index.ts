// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from 'src/store/generic'

// ** Types
import { Params } from 'src/store/scan/state/type'

// ** SDK import
import { StateView } from '@roochnetwork/rooch-sdk'

// ** Fetch Transaction
export const fetchData = createAsyncThunk('state/fetchData', async (params: Params) => {
  params.dispatch(start())

  try {
    let result = await params.provider.getStates(params.accessPath)
    params.dispatch(success(result))

    return result
  } catch (e: any) {
    params.dispatch(error(e.toString()))
  }
})

export const StateViewSlice = CreateGenericSlice({
  name: 'stateView',
  initialState: {
    result: [],
  } as GenericState<StateView | null[]>,
  reducers: {},
})

export default StateViewSlice.reducer

const { start, error, success } = StateViewSlice.actions
