// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useEffect, useMemo, useRef, useState } from 'react'

// ** Next Import
import Link from 'next/link'

// ** MUI Imports
import Card from '@mui/material/Card'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import { DataGrid, GridColDef } from '@mui/x-data-grid'

// ** Utils
import { formatAddress } from 'src/@core/utils/format'
import CardSnippet from '../../../../@core/components/card-snippet'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'
import TextField from '@mui/material/TextField'
import InputAdornment from '@mui/material/InputAdornment'
import Button from '@mui/material/Button'
import Icon from '../../../../@core/components/icon'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

interface CellType {
  row: any
}

// ** Styled components
const LinkStyled = styled(Link)(({ theme }) => ({
  fontSize: '1rem',
  textDecoration: 'none',
  color: theme.palette.primary.main,
}))

const StateList = () => {
  // ** renders client column
  const defaultColumns: GridColDef[] = [
    {
      flex: 0.3,
      minWidth: 90,
      field: 'type',
      headerName: 'Type',
      renderCell: ({ row }: CellType) => (
        <LinkStyled
          href="/"
          onClick={(e) => {
            e.preventDefault()
            handleShowDetail(row)
          }}
        >
          {row.decoded_value.type}
        </LinkStyled>
      ),
    },

    {
      flex: 0.1,
      minWidth: 90,
      field: 'id',
      headerName: 'ID',
      renderCell: ({ row }: CellType) => (
        <Typography sx={{ color: 'text.secondary' }}>
          {formatAddress(row.decoded_value.value.id)}
        </Typography>
      ),
    },

    {
      flex: 0.1,
      minWidth: 90,
      field: 'owner',
      headerName: 'Owner',
      renderCell: ({ row }: CellType) => (
        <Typography sx={{ color: 'text.secondary' }}>
          {formatAddress(row.decoded_value.value.owner)}
        </Typography>
      ),
    },
  ]

  // ** State
  const [accessPath, setAccessPath] = useState<string>('/object/0x1')

  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })
  const [curStateView, setCurStateView] = useState<any | null>(null)

  const mapPageToNextCursor = useRef<{ [page: number]: string | null }>({})
  const [cacheResult, setCacheResult] = useState<{ [page: number]: any[] }>({})
  const [count, setCount] = useState(10)

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1],
      pageSize: paginationModel.pageSize,
    }),
    [paginationModel],
  )

  let { data, isPending, error } = useRoochClientQuery(
    'listStates',
    {
      accessPath: accessPath,
      cursor: queryOptions.cursor,
      limit: paginationModel.pageSize,
    },
    {
      enabled: true,
    },
  )

  useEffect(() => {
    if (!data) {
      return
    }

    if (data.has_next_page) {
      mapPageToNextCursor.current[paginationModel.page] = (data.next_cursor as any) ?? null
    }

    cacheResult[paginationModel.page] = data.data
    setCacheResult({ ...cacheResult })
    const count = Object.values(cacheResult).reduce((count, data) => count + data.length, 0)
    setCount(count)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [paginationModel, data])

  const handleShowDetail = (view: any) => {
    setCurStateView(view)
  }

  return (
    <>
      <Card>
        <CardHeader title="State Filters" />
        <CardContent>
          <TextField
            id="access-path-id"
            label="Access Path"
            fullWidth
            value={accessPath}
            helperText={error?.toString()}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end" sx={{ color: 'text.primary' }}>
                  <Button size="small">
                    <Typography mr={2} color="text.disabled">
                      Enter
                    </Typography>
                    <Icon icon="bx:search" />
                  </Button>
                </InputAdornment>
              ),
            }}
            onChange={(v) => setAccessPath(v.target.value)}
          />
        </CardContent>
      </Card>
      <Card sx={{ mt: 6 }}>
        <DataGrid
          autoHeight
          pagination
          disableColumnMenu={true}
          loading={isPending}
          rowCount={data && data.data ? (data.has_next_page ? count + 1 : count) : 0}
          rows={Object.values(cacheResult)
            .flat()
            .map((row: any) => ({ ...row.state, id: row.state.decoded_value.value.id }))}
          columns={defaultColumns.map((v) => ({
            ...v,
            sortable: false,
          }))}
          paginationModel={paginationModel}
          onPaginationModelChange={setPaginationModel}
        />
      </Card>
      <Typography sx={{ color: 'text.secondary', ml: 6, mt: 4, mb: 4 }}>
        {curStateView ? curStateView.decoded_value.type : 'Current Page Raw Data'}
      </Typography>
      <CardSnippet
        defaultShow={true}
        fullHeight={true}
        codes={[
          {
            code: JSON.stringify(curStateView ?? data?.data, null, 2),
            lng: 'json',
          },
        ]}
      />
    </>
  )
}

export default StateList
