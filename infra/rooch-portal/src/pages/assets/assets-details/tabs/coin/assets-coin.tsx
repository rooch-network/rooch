// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// REACT
import React, { Suspense, useEffect, useState } from 'react'

// SDK
import { BalanceInfoView } from '@roochnetwork/rooch-sdk'
import { useCurrentAccount, useCurrentSession, useRoochClient } from '@roochnetwork/rooch-sdk-kit'

// UI
import { AlertCircle, ArrowLeft, Wallet } from 'lucide-react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { NoData } from '@/components/no-data'
import { Button } from '@/components/ui/button'
import CustomPagination from '@/components/custom-pagination'

// UTIL
import { formatCoin } from '@/utils/format'

// HOOKS
import { useAssetsCoinLogic } from '@/hooks/useAssetsCoinLogic'

const RecipientInput = React.lazy(() => import('@/components/recipient-input'))
const AmountInput = React.lazy(() => import('@/components/amount-input'))

export const AssetsCoin: React.FC = () => {
  const account = useCurrentAccount()
  const sessionKey = useCurrentSession()
  const client = useRoochClient()

  const [modalOpen, setModalOpen] = useState<boolean>(false)

  const handleClose = () => {
    setModalOpen(false)
  }

  const {
    error,
    amount,
    recipient,
    transferLoading,
    data,
    isLoading,
    isError,
    paginationModel,
    handlePageChange,
    handleAmountChange,
    handleRecipientChange,
    handleTransferCoin,
    setSelectedCoin,
    selectedCoin,
  } = useAssetsCoinLogic(sessionKey, handleClose)

  const handleTransferClick = (coin: BalanceInfoView) => {
    setSelectedCoin(coin)
    setModalOpen(true)
  }

  const handleCloseModal = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    if (event.target === event.currentTarget) {
      handleClose()
    }
  }

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

  useEffect(() => {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setModalOpen(false)
      }
    }

    window.addEventListener('keydown', handleEsc)

    return () => {
      window.removeEventListener('keydown', handleEsc)
    }
  }, [])

  if (!account) {
    return (
      <div className="flex flex-col items-center justify-center text-center p-40">
        <Wallet className="w-12 h-12 mb-4 text-zinc-500" />
        <p className="text-xl text-zinc-500 font-semibold">Haven't connected to wallet</p>
        <p className="text-sm text-muted-foreground mt-2">
          Please connect your wallet to view your assets.
        </p>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-40">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="flex flex-col items-center justify-center text-center p-40">
        <AlertCircle className="w-12 h-12 mb-4 text-red-500" />
        <p className="text-xl text-red-500 font-semibold">Error loading data</p>
        <p className="text-sm text-muted-foreground mt-2">
          Something went wrong while fetching the data. Please try again later.
        </p>
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
                <TableCell>
                  {formatCoin(Number(coin.balance), coin.decimals, coin.decimals)}
                </TableCell>
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
              <div className="bg-background dark:bg-zinc-900 rounded-none md:rounded-lg flex flex-col items-start justify-center p-6 w-full h-full md:w-auto md:h-auto overflow-auto max-w-lg mx-auto">
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
                <div className="flex flex-col h-full items-center justify-start gap-6 w-full">
                  {/* Address */}
                  <Suspense fallback={<div>Loading...</div>}>
                    <RecipientInput
                      recipient={recipient}
                      onChange={handleRecipientChange}
                      disabled={transferLoading}
                    />
                    {/* Token + Balance */}
                    <AmountInput
                      amount={amount}
                      onChange={handleAmountChange}
                      selectedCoin={selectedCoin}
                      error={error}
                      disabled={transferLoading}
                    />
                  </Suspense>

                  {/* CTA */}
                  <Button
                    variant="default"
                    size="default"
                    className="w-full mt-6 font-sans gap-1"
                    onClick={() => handleTransferCoin(client, selectedCoin)}
                    disabled={transferLoading || error !== ''}
                  >
                    {transferLoading ? (
                      <div className="flex items-center gap-2">
                        <div className="animate-spin rounded-full h-5 w-5 border-t-2 border-b-2 border-blue-500"></div>
                        <span>Transferring...</span>
                      </div>
                    ) : (
                      <>
                        <span>Send</span>
                        <span className="font-semibold text-blue-400 dark:text-blue-600">
                          {selectedCoin?.name}
                        </span>
                      </>
                    )}
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
