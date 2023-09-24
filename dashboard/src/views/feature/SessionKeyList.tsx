// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useEffect } from 'react'

import { useAuth } from 'src/hooks/useAuth'
import { DataGrid, GridColDef, GridValueGetterParams, GridRenderCellParams } from '@mui/x-data-grid'
import Button from '@mui/material/Button'

// ** Store & Actions Imports
import { fetchData } from 'src/store/session'
import { useAppDispatch, useAppSelector } from 'src/store'

const columns: GridColDef[] = [
  { field: 'authentication_key', headerName: 'Authentication Key', width: 200 },
  {
    field: 'scopes',
    headerName: 'Scopes',
    width: 200,
    valueGetter: (params: GridValueGetterParams) => (params.row.scopes as Array<string>).join(', '),
  },
  {
    field: 'create_time',
    headerName: 'Create Time',
    width: 200,
    valueGetter: (params: GridValueGetterParams) => {
      return new Date(params.row.create_time).toLocaleString()
    },
  },
  {
    field: 'last_active_time',
    headerName: 'Last Active Time',
    width: 200,
    valueGetter: (params: GridValueGetterParams) => {
      return new Date(params.row.last_active_time).toLocaleString()
    },
  },
  {
    field: 'max_inactive_interval',
    headerName: 'Max Inactive Interval',
    width: 200,
    type: 'number',
  },
  {
    field: 'action',
    headerName: 'Action',
    width: 150,
    renderCell: (params: GridRenderCellParams) => (
      <Button
        variant="contained"
        color="secondary"
        onClick={() => handleRemove(params.row.authentication_key)}
      >
        Remove
      </Button>
    ),
  },
]

const handleRemove = (authentication_key: string) => {
  // Handle the remove action here
  console.log(`Remove session key with authentication_key: ${authentication_key}`)
}

export default function SessionKeyList() {
  const auth = useAuth()

  // ** State
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })

  // ** Hooks
  const dispatch = useAppDispatch()
  const { result, status } = useAppSelector((state) => state.session)

  // const clipboard = useClipboard()

  useEffect(() => {
    const defaultAccount = auth.defaultAccount()
    if (!defaultAccount) {
      return
    }

    // Ignore part of request
    if ((!result.has_next_page && status === 'finished') || status === 'loading') {
      return
    }

    dispatch(
      fetchData({
        cursor: paginationModel.page * paginationModel.pageSize,
        limit: paginationModel.pageSize,
        account_address: defaultAccount.address,
        dispatch,
      }),
    )
  }, [dispatch, paginationModel, result, status])

  return (
    <div style={{ height: 400, width: '100%' }}>
      <DataGrid
        rows={status === 'finished' ? result : []}
        loading={status === ('loading' as 'loading')}
        columns={columns}
        checkboxSelection
        pageSizeOptions={[10, 25, 50]}
        paginationModel={paginationModel}
        onPaginationModelChange={setPaginationModel}
      />
    </div>
  )
}
