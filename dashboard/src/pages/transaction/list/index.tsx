// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useEffect, forwardRef } from 'react'

// ** Next Import
import Link from 'next/link'

// ** MUI Imports
import Card from '@mui/material/Card'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import { DataGrid, GridColDef, GridRowId } from '@mui/x-data-grid'

// ** Store & Actions Imports
import { useDispatch, useSelector } from 'react-redux'
import { fetchData } from 'src/store/transaction'

// ** Types Imports
import { RootState, AppDispatch } from 'src/store'
import { ThemeColor } from 'src/@core/layouts/types'

// ** SDK Imports
import { TransactionView } from '@rooch/sdk'

interface InvoiceStatusObj {
  [key: string]: {
    icon: string
    color: ThemeColor
  }
}

interface CustomInputProps {
  dates: Date[]
  label: string
  end: number | Date
  start: number | Date
  setDates?: (value: Date[]) => void
}

interface CellType {
  row: TransactionView
}

// ** Styled components
const LinkStyled = styled(Link)(({ theme }) => ({
  fontSize: '1rem',
  textDecoration: 'none',
  color: theme.palette.primary.main
}))

// ** Vars
const invoiceStatusObj: InvoiceStatusObj = {
  Paid: { color: 'success', icon: 'bx:pie-chart-alt' },
  Sent: { color: 'secondary', icon: 'bx:paper-plane' },
  Downloaded: { color: 'info', icon: 'bx:down-arrow-circle' },
  Draft: { color: 'primary', icon: 'bxs:save' },
  'Past Due': { color: 'error', icon: 'bx:info-circle' },
  'Partial Payment': { color: 'warning', icon: 'bx:adjust' }
}

// ** renders client column

const defaultColumns: GridColDef[] = [
  {
    flex: 0.1,
    field: 'txn_hash',
    minWidth: 80,
    headerName: 'Txn hash',
    renderCell: ({ row }: CellType) => <LinkStyled href='/'>1234</LinkStyled>
  },
  {
    flex: 0.1,
    field: 'method',
    minWidth: 80,
    headerName: 'Method',
    renderCell: ({ row }: CellType) => <LinkStyled href='/'>Transfer</LinkStyled>
  },
  {
    flex: 0.3,
    field: 'age',
    minWidth: 300,
    headerName: 'Age',
    renderCell: ({ row }: CellType) => <LinkStyled href='/'>1010</LinkStyled>
  },
  {
    flex: 0.2,
    minWidth: 125,
    field: 'from',
    headerName: 'From',
    renderCell: ({ row }: CellType) => <Typography sx={{ color: 'text.secondary' }}>0x1234</Typography>
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'to',
    headerName: 'To',
    renderCell: ({ row }: CellType) => <Typography sx={{ color: 'text.secondary' }}>0x5678</Typography>
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'value',
    headerName: 'Value',
    renderCell: ({ row }: CellType) => <Typography sx={{ color: 'text.secondary' }}>0.111</Typography>
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'txn-fee',
    headerName: 'Txn Fee',
    renderCell: ({ row }: CellType) => <Typography sx={{ color: 'text.secondary' }}>0.111</Typography>
  }
]

/* eslint-enable */

const InvoiceList = () => {
  // ** State
  const [dates, setDates] = useState<Date[]>([])
  const [value, setValue] = useState<string>('')
  const [statusValue, setStatusValue] = useState<string>('')
  const [selectedRows, setSelectedRows] = useState<GridRowId[]>([])
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })

  // ** Hooks
  const dispatch = useDispatch<AppDispatch>()
  const store = useSelector((state: RootState) => state.transaction)

  useEffect(() => {
    dispatch(
      fetchData({start:0, end:10}))
    }, [dispatch, statusValue, value, dates])

  return (
    <Card>
      <DataGrid
        autoHeight
        pagination
        rows={store.data}
        columns={defaultColumns}
        pageSizeOptions={[10, 25, 50]}
        paginationModel={paginationModel}
        onPaginationModelChange={setPaginationModel}
        onRowSelectionModelChange={rows => setSelectedRows(rows)}
      />
    </Card>
  )
}

export default InvoiceList
