// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useEffect } from 'react'

// ** Next Import
import Link from 'next/link'

// ** MUI Imports
import Card from '@mui/material/Card'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import { DataGrid, GridColDef } from '@mui/x-data-grid'

// ** Store & Actions Imports
import { fetchData } from 'src/store/scan/transaction'
import { useAppDispatch, useAppSelector } from 'src/store'

// ** SDK Imports
import { TransactionView } from '@rooch/sdk'

interface CellType {
  row: TransactionView
}

// ** Styled components
const LinkStyled = styled(Link)(({ theme }) => ({
  fontSize: '1rem',
  textDecoration: 'none',
  color: theme.palette.primary.main,
}))

// ** renders client column

const defaultColumns: GridColDef[] = [
  {
    flex: 0.1,
    field: 'txn_hash',
    minWidth: 80,
    headerName: 'Txn hash',
    renderCell: ({ row }: CellType) => <LinkStyled href="/">{row.sender}</LinkStyled>,
  },
  {
    flex: 0.1,
    field: 'method',
    minWidth: 80,
    headerName: 'Method',
    renderCell: ({ row }: CellType) => <LinkStyled href="/">{row.sender}</LinkStyled>,
  },
  {
    flex: 0.3,
    field: 'age',
    minWidth: 300,
    headerName: 'Age',
    renderCell: ({ row }: CellType) => <LinkStyled href="/">{row.sender}</LinkStyled>,
  },
  {
    flex: 0.2,
    minWidth: 125,
    field: 'from',
    headerName: 'From',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.sender}</Typography>
    ),
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'to',
    headerName: 'To',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.sender}</Typography>
    ),
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'value',
    headerName: 'Value',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.sender}</Typography>
    ),
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'txn-fee',
    headerName: 'Txn Fee',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.sender}</Typography>
    ),
  },
]

/* eslint-enable */

const InvoiceList = () => {
  // ** State
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })

  // ** Hooks
  const dispatch = useAppDispatch()
  const store = useAppSelector((state) => state.transaction)

  useEffect(() => {
    dispatch(fetchData({ cursor: 0, limit: 10 }))
  }, [dispatch])

  return (
    <Card>
      <DataGrid
        autoHeight
        pagination
        disableColumnMenu={true}
        rows={store.data}
        loading={true}
        columns={defaultColumns.map((v) => ({
          ...v,
          sortable: false,
        }))}
        pageSizeOptions={[10, 25, 50]}
        paginationModel={paginationModel}
        onPaginationModelChange={setPaginationModel}
      />
    </Card>
  )
}

export default InvoiceList
