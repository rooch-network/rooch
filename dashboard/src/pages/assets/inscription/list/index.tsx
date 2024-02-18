// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useEffect, useMemo, useRef, useState } from 'react'

// ** MUI Imports
import Card from '@mui/material/Card'
import Typography from '@mui/material/Typography'
import { DataGrid, GridColDef } from '@mui/x-data-grid'
import Tooltip from '@mui/material/Tooltip'

// ** SDK Imports
import { IndexerStateID, InscriptionStateView } from '@roochnetwork/rooch-sdk'

// ** Utils
import { formatAddress } from 'src/@core/utils/format'
import { useRoochClientQuery, useWalletStore } from '@roochnetwork/rooch-sdk-kit'

interface CellType {
  row: InscriptionStateView
}

// ** renders client column

const defaultColumns: GridColDef[] = [
  {
    flex: 0.1,
    minWidth: 90,
    field: 'mint_time',
    headerName: 'Mint Time',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.created_at}</Typography>
    ),
  },
  {
    flex: 0.1,
    field: 'object_id',
    minWidth: 80,
    headerName: 'Object ID',
    renderCell: ({ row }: CellType) => (
      <Tooltip placement="bottom" sx={{ cursor: 'pointer' }} title={row.object_id}>
        <Typography sx={{ color: 'text.secondary' }}>{formatAddress(row.object_id)}</Typography>
      </Tooltip>
    ),
  },
  {
    flex: 0.1,
    field: 'object_type',
    minWidth: 80,
    headerName: 'Object Type',
    renderCell: ({ row }: CellType) => (
      <Tooltip placement="bottom" sx={{ cursor: 'pointer' }} title={row.object_type}>
        <Typography sx={{ color: 'text.secondary' }}>{row.object_type}</Typography>
      </Tooltip>
    ),
  },
  {
    flex: 0.1,
    field: 'bitcoin_tx',
    minWidth: 80,
    headerName: 'Bitcoin TX',
    renderCell: ({ row }: CellType) => (
      <Tooltip placement="bottom" sx={{ cursor: 'pointer' }} title={row.object_id}>
        <Typography sx={{ color: 'text.secondary' }}>
          {formatAddress(row.value.bitcoin_txid)}
        </Typography>
      </Tooltip>
    ),
  },
  {
    flex: 0.2,
    minWidth: 125,
    field: 'content',
    headerName: 'Content',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>
        {row.value.content_type === 'text/plain;charset=utf-8'
          ? hexToString(row.value.body as any)
          : row.value.content_type}
      </Typography>
    ),
  },
]

function hexToString(hex: string): string {
  if (hex.startsWith('0x')) {
    hex = hex.substring(2)
  }

  let result = ''
  for (let i = 0; i < hex.length; i += 2) {
    const byte = parseInt(hex.substr(i, 2), 16)
    result += String.fromCharCode(byte)
  }

  return result
}

const InscriptionGrad = () => {
  const account = useWalletStore((state) => state.currentAccount)

  // ** State
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateID | null }>({})
  const [cacheResult, setCacheResult] = useState<{ [page: number]: any[] }>({})
  const [count, setCount] = useState(10)

  const cursor = useMemo(
    () => mapPageToNextCursor.current[paginationModel.page - 1],
    [paginationModel],
  )

  let { data, isPending } = useRoochClientQuery(
    'queryInscriptions',
    {
      filter: {
        owner: account?.getAddress() ?? '',
      },
      cursor: cursor,
      limit: paginationModel.pageSize,
      descending_order: true,
    },
    {
      enabled: true,
    },
  )

  useEffect(() => {
    if (data?.next_cursor !== null) {
      mapPageToNextCursor.current[paginationModel.page] = data?.next_cursor ?? null
    }

    if (data) {
      cacheResult[paginationModel.page] = data.data
      setCacheResult({ ...cacheResult })
      const count = Object.values(cacheResult).reduce((count, data) => count + data.length, 0)
      setCount(count)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, paginationModel.page])

  console.log(data)

  return (
    <Card>
      <DataGrid
        autoHeight
        pagination
        disableColumnMenu={true}
        rowCount={data && data.data ? (data.has_next_page ? count + 1 : count) : 0}
        rows={Object.values(cacheResult)
          .flat()
          .map((row: any) => ({ ...row, id: row.object_id }))}
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

export default InscriptionGrad
