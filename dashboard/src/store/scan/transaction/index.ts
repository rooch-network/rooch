// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
// import { Dispatch } from 'redux'
import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'

// ** sdk import
import { JsonRpcProvider, TransactionView } from '@rooch/sdk'

interface DataParams {
  cursor: number
  limit: number
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('Transaction/fetchData', async (params: DataParams) => {
  const jp = new JsonRpcProvider()
  console.log(jp)

  // let result = await jp.getTransactions(params.tx_hashes)

  // console.log(result)

  return ''
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
      // TODO need fixed after transaction RPC refactor
      // state.allData = action.payload
    })
  },
})

export default TransactionSlice.reducer
