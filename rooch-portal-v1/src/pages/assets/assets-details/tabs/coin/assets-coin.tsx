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
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@/components/ui/pagination'
import { NoData } from '@/components/no-data'

import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'

interface Coin {
  id: number
  coin: string
  balance: number
  value: string
}

const coins: Coin[] = [
  { id: 0, coin: 'ROOCH', balance: 288.88, value: '$1,146.98' },
  { id: 1, coin: 'BTC', balance: 2.5, value: '$95,000.00' },
  { id: 2, coin: 'ETH', balance: 10, value: '$30,000.00' },
  { id: 3, coin: 'LTC', balance: 100, value: '$10,000.00' },
  { id: 4, coin: 'XRP', balance: 2000, value: '$1,200.00' },
  { id: 5, coin: 'DOGE', balance: 15000, value: '$4,500.00' },
]

export const AssetsCoin = () => {
  // ** PAGE
  const [currentPage, setCurrentPage] = useState(0)
  const itemsPerPage = 5
  const pageCount = Math.ceil(coins.length / itemsPerPage)
  const currentItems = coins.slice(currentPage * itemsPerPage, (currentPage + 1) * itemsPerPage)
  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) {
      return
    }
    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    })
  }

  // ** Render the `no-data` page while data is empty
  const hasValidData = (coins: Coin[]): boolean => {
    return coins.some((coin) => coin.coin.trim() !== '' && coin.balance !== 0)
  }

  // ** MODAL
  const [modalOpen, setModalOpen] = useState(false)
  const [selectedCoin, setSelectedCoin] = useState<Coin | null>(null)

  const handleTransferClick = (coin: Coin) => {
    setSelectedCoin(coin)
    setModalOpen(true)
  }

  const handleClose = () => {
    setModalOpen(false)
  }

  const handleCloseModal = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    if (event.target === event.currentTarget) {
      handleClose()
    }
  }

  // ** modal 打开时，禁止父组件 scroll
  useEffect(() => {
    if (modalOpen) {
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = ''
    }

    return () => {
      document.body.style.overflow = ''
    }
  }, [modalOpen])

  // ** ESC 关闭 modal
  useEffect(() => {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.keyCode === 27) {
        setModalOpen(false)
      }
    }

    window.addEventListener('keydown', handleEsc)

    return () => {
      window.removeEventListener('keydown', handleEsc)
    }
  }, [])

  if (!hasValidData(coins)) {
    return <NoData />
  }

  return (
    <>
      {/* ASSETS TABLE */}
      <div className="rounded-lg border overflow-hidden w-full">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[120px]">Asset</TableHead>
              <TableHead>Balance</TableHead>
              <TableHead>Value</TableHead>
              <TableHead className="text-right"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {currentItems.map((coin) => (
              <TableRow key={coin.coin}>
                <TableCell className="font-medium">{coin.coin}</TableCell>
                <TableCell>{coin.balance}</TableCell>
                <TableCell>{coin.value}</TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="link"
                    size="sm"
                    className="text-blue-500"
                    onClick={() => handleTransferClick(coin)}
                  >
                    TRANSFER
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      {/* PAGINATION */}
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

      {/* MODAL */}
      {modalOpen && (
        <div className="flex items-center justify-center font-mono">
          <div
            className="fixed inset-0 bg-opacity-70 dark:bg-opacity-75 flex justify-center items-center z-50 bg-black"
            onClick={handleCloseModal}
          >
            <div className="bg-background dark:bg-zinc-900 rounded-none md:rounded-lg flex flex-col items-start justify-center p-6 w-full h-full md:w-auto md:h-auto overflow-auto">
              {/* Back */}
              <div className="mb-4">
                <Button
                  variant="secondary"
                  size="sm"
                  className="h-8 w-14 rounded-2xl bg-accent dark:bg-zinc-800 dark:hover:bg-zinc-700/65"
                  onClick={handleClose}
                >
                  <ArrowLeft className="w-5 h-5 text-muted-foreground dark:text-gray-200" />
                </Button>
              </div>

              {/* Content */}
              <div className="flex flex-col h-full items-center justify-start gap-6">
                {/* Address */}
                <div className="grid w-full max-w-md items-center gap-1.5">
                  <Label htmlFor="address">Send to</Label>
                  <Textarea
                    id="address"
                    placeholder="Enter Address..."
                    className="h-14 resize-none overflow-hidden rounded-2xl bg-gray-50 text-gray-800 w-96"
                    required
                    rows={1}
                  />
                </div>

                {/* Token + Balance */}
                <div className="grid w-full max-w-md items-center gap-1.5">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="address">Amount</Label>
                    <p className="text-xs text-muted-foreground">Balance: 0 {selectedCoin?.coin}</p>
                  </div>
                  <div className="h-14 rounded-2xl bg-zinc-200 dark:bg-zinc-700 w-96 pl-6 flex items-center justify-between relative">
                    <div className="flex items-center justify-center gap-1.5">
                      <span className="bg-white rounded-full p-0.5">
                        <img src="/rooch_black_logo.svg" alt="rooch" className="w-4 h-4" />
                      </span>
                      <p className="text-sm">{selectedCoin?.coin}</p>
                    </div>
                    <Input
                      className="h-10 rounded-2xl bg-gray-50 text-gray-800 w-60 pr-12 mr-2 overflow-hidden border-none"
                      placeholder="0.0"
                      required
                    />
                    <button className="text-blue-500 absolute end-4 font-sans text-xs focus:outline-none focus:ring-0 hover:text-blue-300 transition-all bg-gray-50 h-8 w-8">
                      MAX
                    </button>
                  </div>
                </div>

                {/* CTA */}
                <Button variant="default" size="default" className="w-full mt-6 font-sans">
                  Send {selectedCoin?.coin}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
