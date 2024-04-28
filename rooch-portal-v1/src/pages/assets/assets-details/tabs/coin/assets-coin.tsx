// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import {
  Table,
  TableBody,
  TableCell,
  // TableFooter,
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
import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import { useEffect, useMemo, useRef, useState } from 'react'

export const AssetsCoin = () => {
  const account = useCurrentAccount()

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

  const { data } = useRoochClientQuery('getBalances', {
    address: account?.getRoochAddress() || '',
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

  useEffect(() => {
    if (!data) {
      return
    }

    if (data.has_next_page) {
      mapPageToNextCursor.current[paginationModel.page] = data.next_cursor ?? null
    }
  }, [paginationModel, data])

  return !data || data.data.length === 0 ? (
    <NoData />
  ) : (
    <div className="rounded-lg border overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[120px]">Asset</TableHead>
            <TableHead>Balance</TableHead>
            {/*<TableHead>Value</TableHead>*/}
            <TableHead className="text-right">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {data?.data.map((coin) => (
            <TableRow key={coin.name}>
              {/*<TableCell className="font-medium">{coin.coin}</TableCell>*/}
              <TableCell>{coin.name}</TableCell>
              <TableCell>{coin.balance}</TableCell>
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
                      {/*<DropdownMenuItem onClick={() => {}}>*/}
                      {/*  Swap*/}
                      {/*  <DropdownMenuShortcut>⇧⌘S</DropdownMenuShortcut>*/}
                      {/*</DropdownMenuItem>*/}
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
        <Pagination className="justify-end mt-4">
          <PaginationContent>
            {paginationModel.page !== 0 && (
              <PaginationItem>
                <PaginationPrevious
                  href="#"
                  onClick={() => handlePageChange(paginationModel.page - 1)}
                />
              </PaginationItem>
            )}
            {Array.from(
              { length: Object.values(mapPageToNextCursor.current).length + 1 },
              (_, index) => (
                <PaginationItem key={index}>
                  <PaginationLink
                    href="#"
                    onClick={() => handlePageChange(index)}
                    isActive={index === paginationModel.page}
                  >
                    {index + 1}
                  </PaginationLink>
                </PaginationItem>
              ),
            )}
            {data?.has_next_page && (
              <PaginationItem>
                <PaginationNext
                  href="#"
                  onClick={() => handlePageChange(paginationModel.page + 1)}
                />
              </PaginationItem>
            )}
          </PaginationContent>
        </Pagination>
      </Table>
    </div>
  )
}
