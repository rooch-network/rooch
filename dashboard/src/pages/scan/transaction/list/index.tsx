// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState } from 'react'

// ** Next Import
import Link from 'next/link'

// ** MUI Imports
import Card from '@mui/material/Card'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import { DataGrid, GridColDef } from '@mui/x-data-grid'
import Tooltip from '@mui/material/Tooltip'

// ** SDK Imports
import { TransactionWithInfoView } from '@roochnetwork/rooch-sdk'

// ** Utils
import { formatAddress } from 'src/@core/utils/format'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

interface WrapperTransactionWithInfoView {
  transactionWithInfoView: TransactionWithInfoView
  index: number
}

interface CellType {
  row: WrapperTransactionWithInfoView
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
    minWidth: 90,
    field: 'sequence_order',
    headerName: 'Sequence Order',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>
        {row.transactionWithInfoView.sequence_info.tx_order}
      </Typography>
    ),
  },
  {
    flex: 0.1,
    field: 'tx_hash',
    minWidth: 80,
    headerName: 'Txn hash',
    renderCell: ({ row }: CellType) => (
      <LinkStyled href={`/scan/transaction/detail/${row.index}`}>
        {formatAddress(row.transactionWithInfoView.execution_info.tx_hash)}
      </LinkStyled>
    ),
  },
  {
    flex: 0.1,
    field: 'method',
    minWidth: 80,
    headerName: 'Method',
    renderCell: ({ row }: CellType) => (
      <LinkStyled href="/">
        {row.transactionWithInfoView.transaction.action_type.toUpperCase()}
      </LinkStyled>
    ),
  },
  {
    flex: 0.2,
    minWidth: 125,
    field: 'sender',
    headerName: 'Sender',
    renderCell: ({ row }: CellType) => (
      <Tooltip
        placement="bottom"
        sx={{ cursor: 'pointer' }}
        title={row.transactionWithInfoView.transaction.sender}
      >
        <Typography sx={{ color: 'text.secondary' }}>
          {formatAddress(row.transactionWithInfoView.transaction.sender)}
        </Typography>
        {/*<LinkStyled href="/" onClick={ (event) => {*/}
        {/*    event.preventDefault()*/}
        {/*TODO: copy */}
        {/*}}>{formatAddress(row.transaction.sender)}</LinkStyled>*/}
      </Tooltip>
    ),
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'status',
    headerName: 'Status',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>
        {row.transactionWithInfoView.execution_info.status.type.toUpperCase()}
      </Typography>
    ),
  },
  {
    flex: 0.1,
    minWidth: 90,
    field: 'txn-fee',
    headerName: 'Txn Fee',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>
        {row.transactionWithInfoView.execution_info.gas_used}
      </Typography>
    ),
  },
]

const TransactionList = () => {
  // ** State
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })

  let { data, isPending } = useRoochClientQuery(
    'getTransactions',
    {
      cursor: 0,
      limit: (paginationModel.page + 1) * paginationModel.pageSize,
      descending_order: false,
    },
    {
      enabled: true,
    },
  )

  return (
    <Card>
      <DataGrid
        autoHeight
        pagination
        disableColumnMenu={true}
        rowCount={
          data && data.data ? (data.has_next_page ? data.data.length + 1 : data.data.length) : 0
        }
        rows={
          data && data.data
            ? data.data.map((row, index) => ({
                transactionWithInfoView: row,
                index: index,
                id: row.execution_info.tx_hash,
              }))
            : []
        }
        loading={isPending}
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

export default TransactionList
