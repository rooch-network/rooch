// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useMemo, useRef, useState } from 'react'
import {
  // useCurrentAccount,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit'
import { NoData } from '@/components/no-data'
import { Card, CardHeader } from '@/components/ui/card'
import CustomPagination from '@/components/custom-pagination.tsx'

import { hexToString } from '@/utils/format.ts'
import { AlertCircle } from 'lucide-react'

// test address
// const testAddress = ''

// TODO: 1, loading, 2 pagination, 3 目前只处理了json 铭文，其他类型还需添加ui
export const BitcoinAssetsOrdi = () => {
  // const account = useCurrentAccount()

  // ** PAGINATION
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

  const {
    data: result,
    isLoading,
    isError,
  } = useRoochClientQuery('queryInscriptions', {
    filter: {
      owner: 'bcrt1p79ruqzh9hmmhvaz7x3up3t6pdrmz5hmhz3pfkddxqnfzg0md7upq3jjjev',
    },
    // TODO: 待解决的类型问题
    // @ts-ignore
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

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
