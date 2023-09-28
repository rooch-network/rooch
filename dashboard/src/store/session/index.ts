// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk, Dispatch, AnyAction } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from '../generic'

// ** sdk import
import { IPage, ISessionKey, JsonRpcProvider, ListAnnotatedStateResultPageView } from '@rooch/sdk'

interface DataParams {
  dispatch: Dispatch<AnyAction>
  account_address: string
  cursor: Uint8Array | null
  limit: number
}

const convertToSessionKey = (data: ListAnnotatedStateResultPageView): Array<ISessionKey> => {
  const result = new Array<ISessionKey>()

  for (const state of data.data as any) {
    const moveValue = state?.move_value as any

    if (moveValue) {
      const val = moveValue.value

      result.push({
        authentication_key: val.authentication_key,
        scopes: convertToScopes(val.scopes),
        create_time: parseInt(val.create_time),
        last_active_time: parseInt(val.last_active_time),
        max_inactive_interval: parseInt(val.max_inactive_interval),
      } as ISessionKey)
    }
  }

  return result
}

const convertToScopes = (data: Array<any>): Array<string> => {
  const result = new Array<string>()

  for (const scope of data) {
    result.push(`${scope.module_name}::${scope.module_address}::${scope.function_name}`)
  }

  return result
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('state/fetchData', async (params: DataParams) => {
  params.dispatch(start())

  const jp = new JsonRpcProvider()
  const { account_address, cursor, limit } = params

  try {
    const accessPath = `/resource/${account_address}/0x3::session_key::SessionKeys`
    const state = await jp.getAnnotatedStates(accessPath)
    if (state) {
      const stateView = state as any
      const tableId = stateView[0].state.value

      const accessPath = `/table/${tableId}`
      const pageView = await jp.listAnnotatedStates(accessPath, cursor, limit)

      const result = {
        data: convertToSessionKey(pageView),
        nextCursor: pageView.next_cursor,
        hasNextPage: pageView.has_next_page,
      }

      params.dispatch(success(result))
    }
  } catch (e: any) {
    params.dispatch(error(e.toString()))
  }
})

export const StateSlice = CreateGenericSlice({
  name: 'session',
  initialState: {
    result: {
      data: [],
      nextCursor: null,
      hasNextPage: false,
    },
  } as GenericState<IPage<ISessionKey>>,
  reducers: {},
})

export default StateSlice.reducer

const { start, error, success } = StateSlice.actions
