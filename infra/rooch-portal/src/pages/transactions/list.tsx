// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'

// ** UI Library Components
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@/components/ui/pagination'
import { NoData } from '@/components/no-data'

// ** ROOCH SDK
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

// ** ICONS
import { MenuSquare, ExternalLink } from 'lucide-react'
import { SkeletonList } from '@/components/skeleton-list'
import { formatAddress, formatTimestamp } from '@/utils/format.ts'

export const TransactionsList = () => {
  const navigate = useNavigate()
  const account = useCurrentAddress()

  const [paginationModel, setPaginationModel] = useState({ index: 0, limit: 10 })
  const mapPageToNextCursor = useRef<{ [page: number]: string | null }>({})

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.index - 1]?.toString(),
      limit: paginationModel.limit.toString(),
    }),
    [paginationModel],
  )

  const { data: transactionsResult, isPending } = useRoochClientQuery('queryTransactions', {
    filter: {
      sender: account?.genRoochAddress().toHexAddress() || '',
    },
    cursor: queryOptions.cursor,
    limit: queryOptions.limit,
  })

  useEffect(() => {
    if (!transactionsResult) {
      return
    }

    if (transactionsResult.has_next_page) {
      mapPageToNextCursor.current[paginationModel.index] = transactionsResult.next_cursor ?? null
    }
  }, [paginationModel, transactionsResult])

  const paginate = (index: number): void => {
    console.log(index)
    if (index < 0) {
      return
    }
    setPaginationModel({
      ...paginationModel,
      index,
    })
  }

  const handleToTransactionDetail = (hash: string) => {
    navigate(`/transactions/detail/${hash}`)
  }

  return isPending ? (
    <SkeletonList />
  ) : !transactionsResult || transactionsResult.data.length === 0 ? (
    <NoData />
  ) : (
    <div>
      <div className="rounded-lg border w-full">
        <Table>
          <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[100px]">TXs</TableHead>
              <TableHead>Transaction Hash</TableHead>
              <TableHead>Timestamp</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>Sender</TableHead>
              <TableHead>Gas</TableHead>
              <TableHead className="text-center">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {transactionsResult.data.map((tx) => (
              <TableRow key={tx.execution_info.tx_hash}>
                <TableCell className="font-medium">
                  <Button variant="ghost" size="icon" className="cursor-default bg-accent">
                    <MenuSquare className="h-4 w-4" />
                  </Button>
                </TableCell>
                <TableCell>
                  <div className="flex flex-col md:flex-row items-start md:items-center justify-start gap-1">
                    <span className="hover:no-underline text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all cursor-pointer">
                      <p className="hidden md:block">{tx.execution_info.tx_hash}</p>
                      <p className="md:hidden block">
                        {tx.execution_info.tx_hash.substring(0, 15)}...
                      </p>
                    </span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="text-muted-foreground">
                    {formatTimestamp(Number(tx.transaction.sequence_info.tx_timestamp))}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="text-muted-foreground">
                    {tx.transaction.data.type.toUpperCase()}
                  </Badge>
                </TableCell>
                <TableCell>
                  <div className="flex flex-col md:flex-row items-start md:items-center justify-start gap-1">
                    <span className="hover:no-underline text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all cursor-pointer">
                      {'sender' in tx.transaction.data
                        ? formatAddress(tx.transaction.data.sender)
                        : ''}
                    </span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="default">{tx.execution_info.gas_used}</Badge>
                </TableCell>
                <TableCell className="text-center">
                  <Button
                    variant="link"
                    size="sm"
                    className="text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all"
                    onClick={() => handleToTransactionDetail(tx.execution_info.tx_hash)}
                  >
                    <ExternalLink className="w-4 h-4 mr-1" />
                    View
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
      <Pagination className="mt-4 justify-end">
        <PaginationContent>
          <PaginationItem>
            {paginationModel.index !== 0 ? (
              <PaginationPrevious href="#" onClick={() => paginate(paginationModel.index - 1)} />
            ) : (
              <PaginationPrevious href="#" />
            )}
          </PaginationItem>
          {Array.from({ length: paginationModel.index + 1 }, (_, i) => (
            <PaginationItem key={i}>
              <PaginationLink
                onClick={() => paginate(i)}
                isActive={paginationModel.index === i}
                className="cursor-pointer"
              >
                {i + 1}
              </PaginationLink>
            </PaginationItem>
          ))}
          <PaginationItem>
            <PaginationEllipsis />
          </PaginationItem>
          <PaginationItem>
            {transactionsResult.has_next_page && (
              <PaginationNext href="#" onClick={() => paginate(paginationModel.index + 1)} />
            )}
          </PaginationItem>
        </PaginationContent>
      </Pagination>
    </div>
  )
}
