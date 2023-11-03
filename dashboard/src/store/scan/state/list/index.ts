// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from 'src/store/generic'

// ** Types
import { Params } from 'src/store/scan/state/type'

// ** SDK import
import { StatePageView } from '@rooch/sdk'

interface Options extends Params {
  cursor: Uint8Array | null
  limit: number
}

export const fetchData = createAsyncThunk('state/fetchListData', async (params: Options) => {
  params.dispatch(start())

  try {
    let result = await params.provider.listStates(params.accessPath, params.cursor, params.limit)
    params.dispatch(success(result))

    return result
  } catch (e: any) {
    params.dispatch(error(e.toString()))
  }
})

export const StateSlice = CreateGenericSlice({
  name: 'statePageView',
  initialState: {
    result: {},
  } as GenericState<StatePageView>,
  reducers: {},
})

export default StateSlice.reducer

const { start, error, success } = StateSlice.actions
