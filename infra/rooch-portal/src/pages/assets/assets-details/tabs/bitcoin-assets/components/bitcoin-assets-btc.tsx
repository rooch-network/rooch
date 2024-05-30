// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useMemo, useRef, useState } from 'react'
import {
  useCurrentAccount,
  // useCurrentAccount,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit'

import { NoData } from '@/components/no-data.tsx'
import CustomPagination from '@/components/custom-pagination.tsx'

import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { AlertCircle, Wallet } from 'lucide-react'

// test address
// const testAddress = ''

export const BitcoinAssetsBtc = () => {
  const account = useCurrentAccount()

  // ** PAGINATION
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 1 })
  const mapPageToNextCursor = useRef<{ [page: number]: string | null }>({})
  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) {
      return
    }
    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    })
  }
  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1],
      pageSize: paginationModel.pageSize,
    }),
    [paginationModel],
  )

  // TODO: 1, loading, 2 pagination
  const {
    data: result,
    isLoading,
    isError,
  } = useRoochClientQuery('queryUTXOs', {
    filter: {
      owner: 'bcrt1p79ruqzh9hmmhvaz7x3up3t6pdrmz5hmhz3pfkddxqnfzg0md7upq3jjjev',
    },
    // TODO: 待解决的类型问题
    // @ts-ignore
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

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

  if (isLoading || isError) {
    return (
      <div className="relative p-24">
        <div className="absolute inset-0 bg-inherit bg-opacity-50 flex justify-center items-center">
          {isLoading ? (
            <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
          ) : (
            <div className="flex flex-col items-center justify-center text-center">
              <AlertCircle className="w-12 h-12 mb-4 text-red-500" />
              <p className="text-xl text-red-500 font-semibold">Error loading data</p>
              <p className="text-sm text-muted-foreground mt-2">
                Something went wrong while fetching the data. Please try again later.
              </p>
            </div>
          )}
        </div>
      </div>
    )
  }

  return !result || result.data.length === 0 ? (
    <NoData />
  ) : (
    <>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
        {result?.data.map((data) => (
          <Card
            key={data.object_id}
            className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          >
            <CardHeader className="flex items-center justify-center">
              <h3 className="text-xl md:text-2xl">UTXO #{data.tx_order}</h3>
            </CardHeader>
            <CardContent className="flex items-center justify-center text-sm md:text-base">
              {/*Amount {data.amount.toLocaleString()}*/}
            </CardContent>
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
