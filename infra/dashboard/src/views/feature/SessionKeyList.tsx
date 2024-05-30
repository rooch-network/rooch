// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useRef } from 'react'

import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'
import Button from '@mui/material/Button'

// import Alert from '@mui/material/Alert'
// import Snackbar from '@mui/material/Snackbar'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import DialogTitle from '@mui/material/DialogTitle'
import {
  DataGrid,
  GridColDef,
  GridValueGetterParams,
  GridRenderCellParams,
  GridPaginationModel,
} from '@mui/x-data-grid'

// import { useRoochClientQuery, useWalletStore} from '@roochnetwork/rooch-sdk-kit';

const formatDate = (timestamp: number) => {
  if (timestamp === 0) {
    return `--`
  }

  const date = new Date(timestamp)
  const year = date.getFullYear()
  const month = ('0' + (date.getMonth() + 1)).slice(-2)
  const day = ('0' + date.getDate()).slice(-2)
  const hours = ('0' + date.getHours()).slice(-2)
  const minutes = ('0' + date.getMinutes()).slice(-2)
  const seconds = ('0' + date.getSeconds()).slice(-2)

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}

const PAGE_SIZE = 100

export default function SessionKeyList() {
  const columns: GridColDef[] = [
    {
      field: 'authentication_key',
      flex: 0.2,
      headerName: 'Authentication Key',
    },
    {
      flex: 0.1,
      field: 'scopes',
      headerName: 'Scopes',
      valueGetter: (params: GridValueGetterParams) => {
        return (params.row.scopes as Array<string>).join(', ')
      },
    },
    {
      field: 'max_inactive_interval',
      flex: 0.1,
      headerName: 'Max Inactive Interval',
      type: 'number',
    },
    {
      field: 'last_active_time',
      flex: 0.1,
      headerName: 'Last Active Time',
      valueGetter: (params: GridValueGetterParams) => {
        return formatDate(params.row.last_active_time * 1000)
      },
    },
    {
      field: 'create_time',
      flex: 0.1,
      headerName: 'Create Time',
      valueGetter: (params: GridValueGetterParams) => {
        return formatDate(params.row.create_time * 1000)
      },
    },
    {
      field: 'action',
      headerName: 'Action',
      flex: 0.2,
      align: 'right',
      headerAlign: 'right',
      renderCell: (params: GridRenderCellParams) => (
        <Button
          variant="contained"
          color="secondary"
          onClick={() => showConfirmDeleteDialog(params.row.authentication_key)}
        >
          Remove
        </Button>
      ),
    },
  ]

  const mapPageToNextCursor = useRef<{ [page: number]: Uint8Array | null }>({})

  // ** State
  const [paginationModel, setPaginationModel] = useState({
    page: 0,
    pageSize: PAGE_SIZE,
  })

  // const queryOptions = useMemo(
  //   () => ({
  //     cursor: mapPageToNextCursor.current[paginationModel.page - 1],
  //     pageSize: paginationModel.pageSize,
  //   }),
  //   [paginationModel],
  // )

  // const account = useWalletStore((store) => store.currentAccount)
  // const accessPath = `/resource/${account?.getAddress()}/0x3::session_key::SessionKeys`

  // let { data: state, isPending } = useRoochClientQuery('getStates', accessPath)

  // const tableId = state.decoded_value.value.keys.value.handle
  // let { data, isPending } = useRoochClientQuery('getStates', `/table/${tableId}`)

  const handlePaginationModelChange = (newPaginationModel: GridPaginationModel) => {
    // We have the cursor, we can allow the page transition.
    if (newPaginationModel.page === 0 || mapPageToNextCursor.current[newPaginationModel.page - 1]) {
      setPaginationModel(newPaginationModel)
    }
  }

  const handleConfirmRemove = (authentication_key: string | undefined) => {
    // Todo: how to handle current session key
    setConfirmDeleteDialog({
      open: false,
      authKey: undefined,
    })

    if (!authentication_key) {
      return false
    }

    return false
  }

  const [confirmDeleteDialog, setConfirmDeleteDialog] = useState<{
    open: boolean
    authKey: string | undefined
  }>({
    open: false,
    authKey: undefined,
  })

  const showConfirmDeleteDialog = (authKey: string) => {
    setConfirmDeleteDialog({
      open: true,
      authKey: authKey,
    })
  }

  const handleConfirmDeleteDialogClose = () => {
    setConfirmDeleteDialog({
      open: false,
      authKey: undefined,
    })
  }

  return (
    <Card>
      <CardHeader title="Session Keys" />
      <CardContent>
        <Box sx={{ textAlign: 'right', marginBottom: '10px', mr: '20px' }}>
          <Button variant="contained" color="primary" size="small">
            Refresh
          </Button>
        </Box>
        <DataGrid
          rows={status === 'finished' ? [] : []}
          loading={status === ('loading' as 'loading')}
          columns={columns}
          pageSizeOptions={[10, 25, 50]}
          onPaginationModelChange={handlePaginationModelChange}
          paginationModel={paginationModel}
          autoHeight
        />
        {/*<Snackbar*/}
        {/*  open={!!error}*/}
        {/*  autoHideDuration={6000}*/}
        {/*  anchorOrigin={{ vertical: 'top', horizontal: 'center' }}*/}
        {/*>*/}
        {/*  <Alert severity="error">{error}</Alert>*/}
        {/*</Snackbar>*/}
        <Dialog
          open={confirmDeleteDialog.open}
          onClose={handleConfirmDeleteDialogClose}
          aria-labelledby="alert-dialog-title"
          aria-describedby="alert-dialog-description"
        >
          <DialogTitle id="alert-dialog-title">{'Confirm Deletion'}</DialogTitle>
          <DialogContent>
            <DialogContentText id="alert-dialog-description">
              Are you sure you want to delete this Session Key? Once deleted, the user will be
              logged out and this action cannot be undone.
            </DialogContentText>
          </DialogContent>
          <DialogActions>
            <Button onClick={handleConfirmDeleteDialogClose} color="primary">
              Cancel
            </Button>
            <Button
              onClick={() => handleConfirmRemove(confirmDeleteDialog.authKey)}
              color="primary"
              autoFocus
            >
              Confirm
            </Button>
          </DialogActions>
        </Dialog>
      </CardContent>
    </Card>
  )
}
