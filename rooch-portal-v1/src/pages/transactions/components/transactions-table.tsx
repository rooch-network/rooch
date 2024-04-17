import { useEffect, useState } from 'react'
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
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

// ** ICONS
import { MenuSquare, ExternalLink } from 'lucide-react'
import { LedgerTransactionView, TransactionWithInfoView } from '@roochnetwork/rooch-sdk'
import { SkeletonList } from '@/components/skeleton-list'
import { formatAddress } from '@/utils/format'

export const TransactionsTable = () => {
  const navigate = useNavigate()
  const [txs, setTxs] = useState<TransactionWithInfoView[]>([])
  const [currentPage, setCurrentPage] = useState<number>(1)
  const itemsPerPage = 8

  const indexOfLastItem = currentPage * itemsPerPage
  const indexOfFirstItem = indexOfLastItem - itemsPerPage
  const currentItems = txs.slice(indexOfFirstItem, indexOfLastItem)

  const pageNumbers: number[] = []
  for (let i = 1; i <= Math.ceil(txs.length / itemsPerPage); i++) {
    pageNumbers.push(i)
  }

  const paginate = (pageNumber: number): void => {
    setCurrentPage(pageNumber)
  }

  const { data: transactionsData, isPending } = useRoochClientQuery(
    'getTransactions',
    { cursor: 0, limit: 10, descending_order: false },
    { enabled: true },
  )

  const handleJumpToTxblock = (hash: string) => {
    navigate(`txblock/${hash}`)
  }

  useEffect(() => {
    if (transactionsData && transactionsData.data) {
      setTxs(transactionsData.data as TransactionWithInfoView[])
    }
  }, [transactionsData])

  if (isPending) {
    return <SkeletonList />
  }

  if (!txs || txs.length === 0) {
    return <NoData />
  }

  return (
    <div>
      <div className="rounded-lg border w-full">
        <Table>
          <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[100px]">TXs</TableHead>
              <TableHead>TX Hash</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>Sender</TableHead>
              <TableHead>Gas</TableHead>
              <TableHead className="text-center">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {currentItems.map((tx) => (
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
                    TODO
                  </Badge>
                </TableCell>
                <TableCell>
                  <div className="flex flex-col md:flex-row items-start md:items-center justify-start gap-1">
                    <span className="hover:no-underline text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all cursor-pointer">
                      <p>
                        {formatAddress(
                          (tx.transaction as LedgerTransactionView & { sender: string }).sender,
                        )}
                      </p>
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
                    onClick={() => handleJumpToTxblock(tx.execution_info.tx_hash)}
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
            {currentPage !== 1 ? (
              <PaginationPrevious
                href="#"
                onClick={() => {
                  paginate(currentPage - 1)
                }}
              />
            ) : (
              <PaginationPrevious href="#" />
            )}
          </PaginationItem>
          {pageNumbers.map((number) => (
            <PaginationItem key={number}>
              <PaginationLink
                onClick={() => {
                  paginate(number)
                }}
                isActive={currentPage === number}
                className="cursor-pointer"
              >
                {number}
              </PaginationLink>
            </PaginationItem>
          ))}
          <PaginationItem>
            <PaginationEllipsis />
          </PaginationItem>
          <PaginationItem>
            {currentPage !== pageNumbers.length ? (
              <PaginationNext
                href="#"
                onClick={() => {
                  paginate(currentPage + 1)
                }}
              />
            ) : (
              <PaginationNext href="#" />
            )}
          </PaginationItem>
        </PaginationContent>
      </Pagination>
    </div>
  )
}
