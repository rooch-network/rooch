// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk, Dispatch, AnyAction } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from '../../generic'

// ** sdk import
import { StateView, JsonRpcProvider } from '@rooch/sdk'

interface DataParams {
  dispatch: Dispatch<AnyAction>
  provider: JsonRpcProvider
  accessPath: string
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('state/fetchData', async (params: DataParams) => {
  params.dispatch(start())

  try {
    let result = await params.provider.getStates(params.accessPath)
    params.dispatch(success(result))

    return result
  } catch (e: any) {
    params.dispatch(error(e.toString()))
  }
})

export const StateSlice = CreateGenericSlice({
  name: 'state',
  initialState: {
    result: [],
  } as GenericState<StateView | null[]>,
  reducers: {},
})

export default StateSlice.reducer

const { start, error, success } = StateSlice.actions
