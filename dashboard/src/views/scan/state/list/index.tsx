// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useEffect, useRef, useMemo, useCallback } from 'react'

// ** Next Import
import Link from 'next/link'

// ** MUI Imports
import Card from '@mui/material/Card'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import { DataGrid, GridColDef } from '@mui/x-data-grid'

// ** Store & Actions Imports
import { fetchData } from 'src/store/scan/state/list'
import { useAppDispatch, useAppSelector } from 'src/store'

// ** Utils
import { formatAddress } from 'src/@core/utils/format'
import { useRooch } from '../../../../hooks/useRooch'
import CardSnippet from '../../../../@core/components/card-snippet'
import Grid from '@mui/material/Grid'
import Spinner from '../../../../@core/components/spinner'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'
import TextField from '@mui/material/TextField'
import InputAdornment from '@mui/material/InputAdornment'
import Button from '@mui/material/Button'
import Icon from '../../../../@core/components/icon'

interface CellType {
  row: any
}

// ** Styled components
const LinkStyled = styled(Link)(({ theme }) => ({
  fontSize: '1rem',
  textDecoration: 'none',
  color: theme.palette.primary.main,
}))

const TransactionList = () => {
  // Hook
  const rooch = useRooch()
  const dispatch = useAppDispatch()
  const { result, status, error } = useAppSelector((state) => state.statePageView)

  // ** State
  const [accessPath, setAccessPath] = useState<string>('/object/0x1')
  const [cacheResult, setCacheResult] = useState<{ [page: number]: any[] }>({})

  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })
  const [curStateView, setCurStateView] = useState<any | null>(null)
  const mapPageToNextCursor = useRef<{ [page: number]: Uint8Array | null }>({})
  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1],
      pageSize: paginationModel.pageSize,
    }),
    [paginationModel],
  )
  const [count, setCount] = useState(10)

  useEffect(() => {
    if (status === 'finished') {
      // We add nextCursor when available
      mapPageToNextCursor.current[paginationModel.page] = result.next_cursor
      cacheResult[paginationModel.page] = result.data
      setCacheResult({ ...cacheResult })
      const count = Object.values(cacheResult).reduce((sum, data) => sum + data.length, 0)
      setCount(count)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status])

  useEffect(() => {
    if (cacheResult[paginationModel.page]) {
      return
    }

    dispatch(
      fetchData({
        dispatch,
        provider: rooch.provider!,
        accessPath: accessPath,
        cursor: queryOptions.cursor,
        limit: queryOptions.pageSize,
      }),
    )
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dispatch, paginationModel, rooch])

  const handleSearch = () => {
    dispatch(
      fetchData({
        dispatch,
        provider: rooch.provider!,
        accessPath: accessPath,
        cursor: queryOptions.cursor,
        limit: queryOptions.pageSize,
      }),
    )
  }

  useEffect(() => {
    handleSearch()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dispatch])

  // Handle shortcut keys keyup events
  const handleKeyUp = useCallback(
    (event: KeyboardEvent) => {
      if (event.keyCode === 13) {
        dispatch(
          fetchData({
            dispatch,
            provider: rooch.provider!,
            accessPath: accessPath,
            cursor: queryOptions.cursor,
            limit: queryOptions.pageSize,
          }),
        )
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [dispatch, accessPath, rooch],
  )

  useEffect(() => {
    document.addEventListener('keyup', handleKeyUp)

    return () => {
      document.removeEventListener('keyup', handleKeyUp)
    }
  }, [handleKeyUp])

  const handleShowDetail = (view: any) => {
    setCurStateView(view)
  }

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

  console.log(status, status === 'loading')

  return (
    <Grid container spacing={6}>
      <Grid item xs={12}>
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
                  <InputAdornment
                    position="end"
                    sx={{ color: 'text.primary' }}
                    onClick={handleSearch}
                  >
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
      </Grid>
      {status === 'loading' ? (
        <Grid item xs={12}>
          <Spinner />
        </Grid>
      ) : (
        <>
          <Grid item xs={12}>
            <Card>
              <DataGrid
                pagination
                disableColumnMenu={true}
                rowCount={status === 'finished' ? (result.has_next_page ? count + 1 : count) : 0}
                rows={
                  status === 'finished'
                    ? Object.values(cacheResult)
                        .flat()
                        .map((row: any) => ({ ...row, id: row.decoded_value.value.id }))
                    : []
                }
                columns={defaultColumns.map((v) => ({
                  ...v,
                  sortable: false,
                }))}
                paginationModel={paginationModel}
                onPaginationModelChange={setPaginationModel}
              />
            </Card>
          </Grid>
          <Typography sx={{ color: 'text.secondary', ml: 6, mt: 4 }}>
            {curStateView ? curStateView.decoded_value.type : 'Current Page Raw Data'}
          </Typography>
          <Grid item xs={12}>
            <CardSnippet
              defaultShow={true}
              fullHeight={true}
              codes={[
                {
                  code: JSON.stringify(curStateView ?? cacheResult[paginationModel.page], null, 2),
                  lng: 'json',
                },
              ]}
            />
          </Grid>
        </>
      )}
    </Grid>
  )
}

export default TransactionList
