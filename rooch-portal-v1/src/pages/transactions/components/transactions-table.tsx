import { useState } from 'react'

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

// ** ROOCH SDK
// import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

// ** ICONS
import { MenuSquare, ExternalLink } from 'lucide-react'

interface TransactionsProps {
  type: string
  txHash: string
  from: string
  to: string
  amount: number
  asset: string
  date: string
}

const txs: TransactionsProps[] = [
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0xfa3cd16c57a8a0288f16073bf0afbcf9aa1192c532c8fe65712f333282104068',
    from: '0x1D731fDc4411B961B7067318E549f6A36bf518F1',
    to: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.00188,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
  {
    type: 'transaction',
    txHash: '0x79be8a1ffe4a0f65b205905352acf4a2b521490fa45600df6ebe62fb24e05b97',
    from: '0x388C818CA8B9251b393131C08a736A67ccB19297',
    to: '0xa83114A443dA1CecEFC50368531cACE9F37fCCcb',
    amount: 0.232345,
    asset: 'ETH',
    date: '2023/10/01 14:30',
  },
]

export const TransactionsTable = () => {
  const [currentPage, setCurrentPage] = useState<number>(1)
  const itemsPerPage = 8

  // ** Calculate the indices for the current page
  const indexOfLastItem = currentPage * itemsPerPage
  const indexOfFirstItem = indexOfLastItem - itemsPerPage

  // ** Slice the txs array to get only the items for the current page
  const currentItems = txs.slice(indexOfFirstItem, indexOfLastItem)

  // ** Generate the page numbers for pagination
  const pageNumbers: number[] = []
  for (let i = 1; i <= Math.ceil(txs.length / itemsPerPage); i++) {
    pageNumbers.push(i)
  }

  // ** Function to change the page
  const paginate = (pageNumber: number): void => {
    setCurrentPage(pageNumber)
  }

  // ** Fetch Transactions with SDK
  // const { data: transactionsData, isPending } = useRoochClientQuery(
  //   'getTransactions',
  //   {
  //     cursor: 0,
  //     limit: 10,
  //     descending_order: false,
  //   },
  //   {
  //     enabled: true,
  //   },
  // )

  // if (isPending) {
  //   return <div>Loading...</div>
  // }

  // if (!transactionsData) {
  //   return <div>No transactions data available.</div>
  // }

  // console.log(transactionsData.data)

  return (
    <div>
      <div className="rounded-lg border w-full">
        <Table>
          <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[100px]">TXs</TableHead>
              <TableHead>TX Hash/Date</TableHead>
              <TableHead>From/to</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead className="text-center">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {currentItems.map((tx) => (
              <TableRow key={tx.txHash}>
                <TableCell className="font-medium">
                  <Button variant="ghost" size="icon" className="cursor-default bg-accent">
                    <MenuSquare className="h-4 w-4" />
                  </Button>
                </TableCell>
                <TableCell>
                  <div className="flex flex-col md:flex-row items-start md:items-center justify-start gap-1">
                    <span className="hover:no-underline text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all cursor-pointer">
                      {/* 前十五位 */}
                      {tx.txHash.substring(0, 15)}...
                    </span>
                    <span className="text-xs text-muted-foreground">{tx.date}</span>
                  </div>
                </TableCell>
                <TableCell>
                  <div className="flex flex-col md:flex-row items-start md:items-center justify-start gap-1">
                    <span className="hover:no-underline text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all cursor-pointer flex items-center justify-start gap-1">
                      <p className="text-primary">From</p>
                      {/* 前八位和后八位，中间用...省略 */}
                      {tx.from.substring(0, 8)}...
                      {tx.from.substring(tx.to.length - 8)}
                    </span>
                    <span className="hover:no-underline text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all cursor-pointer flex items-center justify-start gap-1">
                      <p className="text-primary">To</p>
                      {/* 前八位和后八位，中间用...省略 */}
                      {tx.to.substring(0, 8)}...
                      {tx.to.substring(tx.to.length - 8)}
                    </span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="text-muted-foreground">
                    {tx.type}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge variant="default">
                    {tx.amount}
                    {tx.asset}
                  </Badge>
                </TableCell>
                <TableCell className="text-center">
                  <Button
                    variant="link"
                    size="sm"
                    className="text-blue-400 hover:text-blue-500 dark:text-blue-300 dark:hover:text-blue-200 transition-all"
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
      {/* Pagination Functionality */}
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
