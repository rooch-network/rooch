// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { Dispatch } from 'redux'
import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'

// ** sdk import
import { JsonRpcProvider, TransactionView } from '@rooch/sdk'

interface DataParams {
  start: number,
  end: number
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('Transaction/fetchData', async (params: DataParams) => {

  const jp = new JsonRpcProvider()
  let result = await jp.getTransactionByIndex(params.start, params.end)

  console.log(result)

  return result
})

export const TransactionSlice = createSlice({
  name: 'Transaction',
  initialState: {
    data: [] as TransactionView[],
    total: 1,
    params: {},
    allData: [] as TransactionView[],
  },
  reducers: {},
  extraReducers: (builder) => {
    builder.addCase(fetchData.fulfilled, (state, action) => {
      state.allData = action.payload
    })
  },
})

export default TransactionSlice.reducer
