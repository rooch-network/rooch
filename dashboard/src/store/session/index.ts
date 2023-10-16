// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Redux Imports
import { createAsyncThunk, Dispatch, AnyAction } from '@reduxjs/toolkit'

// ** Store Generic Imports
import { CreateGenericSlice, GenericState } from '../generic'

// ** sdk import
import {
  IAccount,
  IPage,
  ISessionKey,
  JsonRpcProvider,
  StatePageView,
  parseRoochErrorSubStatus,
  getErrorCategoryName,
} from '@rooch/sdk'

interface DataParams {
  dispatch: Dispatch<AnyAction>
  provider: JsonRpcProvider
  account_address: string
  cursor: Uint8Array | null
  limit: number
}

interface RemoveParams {
  dispatch: Dispatch<AnyAction>
  account: IAccount
  auth_key: string
  refresh: () => void
}

const convertToSessionKey = (data: StatePageView): Array<ISessionKey> => {
  const result = new Array<ISessionKey>()

  for (const state of data.data as any) {
    const moveValue = state?.decoded_value as any

    if (moveValue) {
      const val = moveValue.value

      result.push({
        id: val.authentication_key,
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
    const val = scope.value
    result.push(`${simplifyHex(val.module_address)}::${val.module_name}::${val.function_name}`)
  }

  return result
}

function simplifyHex(hex: string): string {
  return '0x' + BigInt(hex).toString(16)
}

// ** Fetch Transaction
export const fetchData = createAsyncThunk('state/fetchData', async (params: DataParams) => {
  params.dispatch(start())

  const { account_address, cursor, limit } = params

  try {
    const accessPath = `/resource/${account_address}/0x3::session_key::SessionKeys`
    const stateResult = await params.provider.getStates(accessPath)
    if (stateResult) {
      const stateView = stateResult as any
      if (stateView && stateView.length > 0 && stateView[0]) {
        const tableId = stateView[0].state.value

        const accessPath = `/table/${tableId}`
        const pageView = await params.provider.listStates(accessPath, cursor, limit)

        const result = {
          data: convertToSessionKey(pageView),
          nextCursor: pageView.next_cursor,
          hasNextPage: pageView.has_next_page,
        }

        params.dispatch(success(result))
      } else {
        params.dispatch(success({ data: [], nextCursor: null, hasNextPage: false }))
      }
    }
  } catch (e: any) {
    const subStatus = parseRoochErrorSubStatus(e.message)
    if (subStatus) {
      params.dispatch(
        error(
          'list session keys fail, error category: ' +
            getErrorCategoryName(subStatus.category) +
            ', reason: ' +
            subStatus.reason,
        ),
      )
    } else {
      params.dispatch(error(`list session keys fail, reason: ${e.message}`))
    }

    setTimeout(() => {
      params.dispatch(error(null))
    }, 5000)
  }
})

export const removeRow = createAsyncThunk('state/removeRow', async (params: RemoveParams) => {
  params.dispatch(start())

  const { auth_key } = params

  try {
    const tx = await params.account.removeSessionKey(auth_key)
    console.log('remove_session_key_entry tx:', tx)

    params.refresh()
  } catch (e: any) {
    const subStatus = parseRoochErrorSubStatus(e.message)
    if (subStatus) {
      params.dispatch(
        error(
          'remove session key fail, error category: ' +
            getErrorCategoryName(subStatus.category) +
            ', reason: ' +
            subStatus.reason,
        ),
      )
    } else {
      params.dispatch(error(`remove session key fail, reason: ${e.message}`))
    }

    setTimeout(() => {
      params.dispatch(error(null))
    }, 5000)
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
