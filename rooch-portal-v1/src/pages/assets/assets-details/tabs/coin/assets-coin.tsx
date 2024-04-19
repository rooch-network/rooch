import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@/components/ui/pagination'
import { NoData } from '@/components/no-data'
import { Button } from '@/components/ui/button'

import { GripVerticalIcon } from 'lucide-react'
import { useState } from 'react'

interface Coin {
  coin: string
  balance: number
  value: string
}

const coins: Coin[] = [
  { coin: 'ROOCH', balance: 288.88, value: '$1,146.98' },
  { coin: 'BTC', balance: 2.5, value: '$95,000.00' },
  { coin: 'ETH', balance: 10, value: '$30,000.00' },
  { coin: 'LTC', balance: 100, value: '$10,000.00' },
  { coin: 'XRP', balance: 2000, value: '$1,200.00' },
  { coin: 'DOGE', balance: 15000, value: '$4,500.00' },
]

export const AssetsCoin = () => {
  const [currentPage, setCurrentPage] = useState(0)
  const itemsPerPage = 5

  const pageCount = Math.ceil(coins.length / itemsPerPage)
  const currentItems = coins.slice(currentPage * itemsPerPage, (currentPage + 1) * itemsPerPage)

  const handlePageChange = (selectedPage: number) => {
    setCurrentPage(selectedPage)
  }

  const hasValidData = (coins: Coin[]): boolean => {
    return coins.some((coin) => coin.coin.trim() !== '' && coin.balance !== 0)
  }

  if (!hasValidData(coins)) {
    return <NoData />
  }

  return (
    <>
      <div className="rounded-lg border overflow-hidden w-full">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[120px]">Asset</TableHead>
              <TableHead>Balance</TableHead>
              <TableHead>Value</TableHead>
              <TableHead className="text-right">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {currentItems.map((coin) => (
              <TableRow key={coin.coin}>
                <TableCell className="font-medium">{coin.coin}</TableCell>
                <TableCell>{coin.balance}</TableCell>
                <TableCell>{coin.value}</TableCell>
                <TableCell className="text-right">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button size="icon" variant="ghost" className="hover:rounded-lg">
                        <GripVerticalIcon className="w-5 h-5" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent className="w-56">
                      <DropdownMenuLabel>Action</DropdownMenuLabel>
                      <DropdownMenuSeparator />
                      <DropdownMenuGroup>
                        <DropdownMenuItem onClick={() => {}}>
                          Transfer
                          <DropdownMenuShortcut>⇧⌘F</DropdownMenuShortcut>
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => {}}>
                          Swap
                          <DropdownMenuShortcut>⇧⌘S</DropdownMenuShortcut>
                        </DropdownMenuItem>
                      </DropdownMenuGroup>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
      <Pagination className="justify-end mt-4">
        <PaginationContent>
          <PaginationItem>
            <PaginationPrevious
              href="#"
              onClick={() => handlePageChange(Math.max(0, currentPage - 1))}
            />
          </PaginationItem>
          {Array.from({ length: pageCount }, (_, index) => (
            <PaginationItem key={index}>
              <PaginationLink
                href="#"
                onClick={() => handlePageChange(index)}
                isActive={index === currentPage}
              >
                {index + 1}
              </PaginationLink>
            </PaginationItem>
          ))}
          {currentPage < pageCount - 1 && (
            <PaginationItem>
              <PaginationNext
                href="#"
                onClick={() => handlePageChange(Math.min(pageCount - 1, currentPage + 1))}
              />
            </PaginationItem>
          )}
        </PaginationContent>
      </Pagination>
    </>
  )
}
