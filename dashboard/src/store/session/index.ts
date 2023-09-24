// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk, Dispatch, AnyAction } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from '../generic'

// ** sdk import
import { AnnotatedStateView, JsonRpcProvider } from '@rooch/sdk'

interface ISessionKey {
  authentication_key: string
  scopes: Array<string>
  create_time: number
  last_active_time: number
  max_inactive_interval: number
}

interface DataParams {
  dispatch: Dispatch<AnyAction>
  account_address: string
  cursor: number
  limit: number
}

const convertStatesToSessionKeys = (state: AnnotatedStateView | null[]): Array<ISessionKey> => {
  return []
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('state/fetchData', async (params: DataParams) => {
  params.dispatch(start())

  const jp = new JsonRpcProvider()

  try {
    const accessPath = `/resource/${params.account_address}/SessionKeys`
    let result = await jp.getAnnotatedStates(accessPath)
    params.dispatch(success(convertStatesToSessionKeys(result)))

    return result
  } catch (e: any) {
    params.dispatch(error(e.toString()))
  }
})

export const StateSlice = CreateGenericSlice({
  name: 'session',
  initialState: {
    result: [],
  } as GenericState<ISessionKey[]>,
  reducers: {},
})

export default StateSlice.reducer

const { start, error, success } = StateSlice.actions
