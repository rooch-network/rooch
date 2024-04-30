// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// import { NoData } from '@/components/no-data'
import { useState } from 'react'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import PaginationComponent from '@/components/custom-pagination.tsx'
import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
// import { hexToString } from '@/utils/format.ts'

// test address
// const testAddress = ''

interface MockData {
  id: number
  amount: number
}

const mockData: MockData[] = [
  { id: 123, amount: 1234 },
  { id: 456, amount: 2468 },
  { id: 789, amount: 3690 },
  { id: 1011, amount: 4812 },
  { id: 1011, amount: 4812 },
  { id: 1011, amount: 4812 },
]

// TODO: 1, loading, 2 pagination, 3 目前只处理了json 铭文，其他类型还需添加ui
export const IndexedAssetsOrdi = () => {
  // ** FILL DATA @Sven
  const account = useCurrentAccount()

  const { data: result } = useRoochClientQuery('queryInscriptions', {
    filter: {
      owner: 'bcrt1p79ruqzh9hmmhvaz7x3up3t6pdrmz5hmhz3pfkddxqnfzg0md7upq3jjjev',
    },
  })

  console.log(account)
  console.log(result)

  // ** PAGINATION
  const [currentPage, setCurrentPage] = useState(1)
  const [itemsPerPage] = useState(4)
  const indexOfLastItem = currentPage * itemsPerPage
  const indexOfFirstItem = indexOfLastItem - itemsPerPage
  const currentItems = mockData.slice(indexOfFirstItem, indexOfLastItem)
  const totalPages = Math.ceil(mockData.length / itemsPerPage)

  // return !result || result.data.length === 0 ? (
  //   <NoData />
  // ) : (
  return (
    <>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
        {currentItems.map((item) => (
          <Card
            key={item.id}
            className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          >
            <CardHeader className="flex items-center justify-center">
              <h3 className="text-xl md:text-2xl">
                Inscriptions #{/*{item.value.content_type === 'text/plain;charset=utf-8'*/}
                {/*  ? hexToString(item.value.body as unknown as string)*/}
                {/*  : item.value.content_type}*/}
                {item.amount}
              </h3>
            </CardHeader>
            <CardContent className="flex items-center justify-center text-sm md:text-base">
              {/* Amount {data.amount.toLocaleString()} */}
            </CardContent>
          </Card>
        ))}
      </div>

      <PaginationComponent
        currentPage={currentPage}
        totalPages={totalPages}
        onPageChange={setCurrentPage}
      />
    </>
  )
}
