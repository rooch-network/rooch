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
import { IndexerStateID, UTXOStateView } from '@roochnetwork/rooch-sdk'

import { useWalletStore } from '@roochnetwork/rooch-sdk-kit'

// ** Utils
import { formatAddress } from 'src/@core/utils/format'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

interface CellType {
  row: UTXOStateView
}

// ** renders client column

const defaultColumns: GridColDef[] = [
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
          {formatAddress((row.value?.bitcoin_txid as any) ?? '')}
        </Typography>
      </Tooltip>
    ),
  },
  {
    flex: 0.2,
    minWidth: 125,
    field: 'seals',
    headerName: 'Seals',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.value?.seals}</Typography>
    ),
  },
  {
    flex: 0.2,
    minWidth: 125,
    field: 'value',
    headerName: 'Value',
    renderCell: ({ row }: CellType) => (
      <Typography sx={{ color: 'text.secondary' }}>{row.value?.value}</Typography>
    ),
  },
]

const UTXOList = () => {
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
    'queryUTXOs',
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

export default UTXOList
