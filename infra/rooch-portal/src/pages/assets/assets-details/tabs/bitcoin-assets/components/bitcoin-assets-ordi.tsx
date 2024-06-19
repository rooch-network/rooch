// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useEffect, useMemo, useRef, useState } from 'react'
import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import { NoData } from '@/components/no-data'
import { Card, CardHeader } from '@/components/ui/card'
import CustomPagination from '@/components/custom-pagination.tsx'
import { hexToString } from '@/utils/format.ts'
import { AlertCircle, Wallet } from 'lucide-react'

import { CursorType } from '@/common/interface'
import type { IndexerStateID } from '@roochnetwork/rooch-sdk'

export const BitcoinAssetsOrdi: React.FC = () => {
  const account = useCurrentAccount()

  // ** PAGINATION
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 1 })
  const mapPageToNextCursor = useRef<{ [page: number]: CursorType }>({})

  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) return

    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    })
  }

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1] || null,
      pageSize: paginationModel.pageSize,
    }),
    [paginationModel],
  )

  const {
    data: result,
    isLoading,
    isError,
  } = useRoochClientQuery('queryInscriptions', {
    filter: 'all',
    cursor: queryOptions.cursor as IndexerStateID | null,
    limit: queryOptions.pageSize,
  })

  console.log(result)

  useEffect(() => {
    if (result && result.has_next_page) {
      mapPageToNextCursor.current[paginationModel.page] = (result.next_cursor as CursorType) || null
    }
  }, [result, paginationModel.page])

  if (!account) {
    return (
      <div className="flex flex-col items-center justify-center text-center p-40">
        <Wallet className="w-12 h-12 mb-4 text-zinc-500" />
        <p className="text-xl text-zinc-500 font-semibold">Haven't connected to wallet</p>
        <p className="text-sm text-muted-foreground mt-2">
          Please connect your wallet to view your assets.
        </p>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="relative p-24">
        <div className="absolute inset-0 bg-inherit bg-opacity-50 flex justify-center items-center">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <div className="relative p-24">
        <div className="absolute inset-0 bg-inherit bg-opacity-50 flex justify-center items-center">
          <div className="flex flex-col items-center justify-center text-center">
            <AlertCircle className="w-12 h-12 mb-4 text-red-500" />
            <p className="text-xl text-red-500 font-semibold">Error loading data</p>
            <p className="text-sm text-muted-foreground mt-2">
              Something went wrong while fetching the data. Please try again later.
            </p>
          </div>
        </div>
      </div>
    )
  }

  console.log(result)

  if (!result || result.data.length === 0) {
    return <NoData />
  }

  return (
    <>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
        {result.data.map((item) => (
          <Card
            key={item.object_id}
            className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          >
            <CardHeader className="flex items-center justify-center">
              <h3 className="text-xl md:text-2xl">
                Inscriptions #
                {item.value.content_type === 'text/plain;charset=utf-8'
                  ? hexToString(item.value.body as unknown as string)
                  : item.value.content_type}
              </h3>
            </CardHeader>
            {/*<CardContent className="flex items-center justify-center text-sm md:text-base">*/}
            {/*  Amount {data.amount.toLocaleString()}*/}
            {/*</CardContent>*/}
          </Card>
        ))}
      </div>

      <CustomPagination
        currentPage={paginationModel.page}
        hasNextPage={!!result?.has_next_page}
        onPageChange={handlePageChange}
      />
    </>
  )
}
