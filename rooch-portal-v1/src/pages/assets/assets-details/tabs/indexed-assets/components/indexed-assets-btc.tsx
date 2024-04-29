// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// import { NoData } from '@/components/no-data'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
// import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import { useState } from 'react'

import PaginationComponent from '@/components/custom-pagination.tsx'

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

export const IndexedAssetsBTC = () => {
  // ** PAGINATION
  const [currentPage, setCurrentPage] = useState(1)
  const [itemsPerPage] = useState(4)
  const indexOfLastItem = currentPage * itemsPerPage
  const indexOfFirstItem = indexOfLastItem - itemsPerPage
  const currentItems = mockData.slice(indexOfFirstItem, indexOfLastItem)
  const totalPages = Math.ceil(mockData.length / itemsPerPage)

  // ** FILL DATA @Sven
  // const account = useCurrentAccount()

  // TODO: 1, loading, 2 pagination
  // const { data: result } = useRoochClientQuery('queryUTXOs', {
  //   filter: {
  //     owner: 'bcrt1p79ruqzh9hmmhvaz7x3up3t6pdrmz5hmhz3pfkddxqnfzg0md7upq3jjjev',
  //   },
  // })

  // return !result || result.data.length === 0 ? (
  //   <NoData />
  // ) : (

  return (
    <>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
        {currentItems.map((data) => (
          <Card
            key={data.id}
            className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          >
            <CardHeader className="flex items-center justify-center">
              <h3 className="text-xl md:text-2xl">UTXO #{data.amount}</h3>
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
