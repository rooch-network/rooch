// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk, Dispatch, AnyAction } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from '../../generic'

// ** sdk import
import { JsonRpcProvider, TransactionResultPageView } from '@rooch/sdk'

interface DataParams {
  dispatch: Dispatch<AnyAction>
  cursor: number,
  limit: number
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('state/fetchData', async (params: DataParams) => {
  params.dispatch(start())

  const jp = new JsonRpcProvider()

  try {
    let result = await jp.getTransactionsByOrder(params.cursor, params.limit)
    params.dispatch(success(result))

    return result
  } catch (e: any) {
    params.dispatch(error(e.toString()))
  }
})

export const TransactionSlice = CreateGenericSlice({
  name: 'state',
  initialState: {
    result: {},
  } as GenericState<TransactionResultPageView>,
  reducers: {},
})

export default TransactionSlice.reducer

const { start, error, success } = TransactionSlice.actions
