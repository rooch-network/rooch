// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useMemo, useRef, useState } from 'react'
import {
  // useCurrentAccount,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit'

import { NoData } from '@/components/no-data.tsx'
import CustomPagination from '@/components/custom-pagination.tsx'

import { Card, CardContent, CardHeader } from '@/components/ui/card'

// test address
// const testAddress = ''

export const IndexedAssetsBTC = () => {
  // ** FILL DATA
  // const account = useCurrentAccount()

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
  const { data: result } = useRoochClientQuery('queryUTXOs', {
    filter: {
      owner: 'bcrt1p79ruqzh9hmmhvaz7x3up3t6pdrmz5hmhz3pfkddxqnfzg0md7upq3jjjev',
    },
    // TODO: 待解决的类型问题
    // @ts-ignore
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

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
