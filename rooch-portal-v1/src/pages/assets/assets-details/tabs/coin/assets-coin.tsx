// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useEffect, useMemo, useRef, useState } from 'react'
import { BalanceInfoView } from '@roochnetwork/rooch-sdk'
import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

import { ArrowLeft } from 'lucide-react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { NoData } from '@/components/no-data'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { LoadingSpinner } from '@/components/loading-spinner.tsx'

import CustomPagination from '@/components/custom-pagination.tsx'

export const AssetsCoin = () => {
  const account = useCurrentAccount()

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

  const { data, isLoading, isError } = useRoochClientQuery('getBalances', {
    address: account?.getRoochAddress() || '',
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

  // ** MODAL
  const [modalOpen, setModalOpen] = useState(false)
  const [selectedCoin, setSelectedCoin] = useState<BalanceInfoView | null>(null)

  const handleTransferClick = (coin: BalanceInfoView) => {
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

  useEffect(() => {
    if (!data) {
      return
    }

    if (data.has_next_page) {
      mapPageToNextCursor.current[paginationModel.page] = data.next_cursor ?? null
    }
  }, [paginationModel, data])

  if (isLoading || isError) {
    return (
      <div className="relative my-12">
        <div className="absolute inset-0 bg-black bg-opacity-50 flex justify-center items-center">
          {isLoading ? <LoadingSpinner /> : <div>Error loading data</div>}
        </div>
      </div>
    )
  }

  return !data || data.data.length === 0 ? (
    <NoData />
  ) : (
    <>
      <div className="rounded-lg border overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[175px]">Asset</TableHead>
              <TableHead>Balance</TableHead>
              <TableHead className="text-right">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.data.map((coin) => (
              <TableRow key={coin.name}>
                <TableCell>{coin.name}</TableCell>
                <TableCell>{coin.balance}</TableCell>
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
                      className="h-14 resize-none overflow-hidden rounded-2xl bg-gray-50 dark:bg-gray-200 text-gray-800 w-96"
                      required
                      rows={1}
                    />
                  </div>

                  {/* Token + Balance */}
                  <div className="grid w-full max-w-md items-center gap-1.5">
                    <div className="flex items-center justify-between">
                      <Label htmlFor="address">Amount</Label>
                      <p className="text-xs text-muted-foreground">
                        <span>Balance: </span>
                        <span className="font-semibold text-blue-600 dark:text-blue-400">
                          {selectedCoin?.balance}
                        </span>
                      </p>
                    </div>
                    <div className="h-14 rounded-2xl bg-zinc-200 dark:bg-zinc-700 w-96 pl-6 flex items-center justify-between relative">
                      <div className="flex items-center justify-center gap-1.5">
                        <span className="bg-white rounded-full p-0.5">
                          <img src="/rooch_black_logo.svg" alt="rooch" className="w-4 h-4" />
                        </span>
                        <p className="text-sm">{selectedCoin?.name}</p>
                      </div>
                      <Input
                        className="h-10 rounded-2xl bg-gray-50 dark:bg-gray-200 text-gray-800 w-48 pr-8 mr-2 overflow-hidden border-none"
                        placeholder="0.0"
                        required
                      />
                      <button className="text-blue-500 absolute end-4 font-sans text-xs focus:outline-none focus:ring-0 hover:text-blue-300 transition-all bg-gray-50 h-8 w-8 dark:bg-gray-200 rounded-2xl">
                        MAX
                      </button>
                    </div>
                  </div>

                  {/* CTA */}
                  <Button variant="default" size="default" className="w-full mt-6 font-sans gap-1">
                    <span>Send</span>
                    <span className="font-semibold text-blue-400 dark:text-blue-600">
                      {selectedCoin?.name}
                    </span>
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      <CustomPagination
        currentPage={paginationModel.page}
        hasNextPage={!!data?.has_next_page}
        onPageChange={handlePageChange}
      />
    </>
  )
}
