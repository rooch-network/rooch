// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useEffect, useMemo, useRef, useState } from 'react'
import { BalanceInfoView } from '@roochnetwork/rooch-sdk'
import {
  useCurrentAccount,
  useCurrentSession,
  useRoochClientQuery,
  useTransferCoin,
} from '@roochnetwork/rooch-sdk-kit'
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
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import CustomPagination from '@/components/custom-pagination.tsx'
import { formatCoin } from '@/utils/format.ts'
import { useToast } from '@/components/ui/use-toast'
import { ToastAction } from '@/components/ui/toast'

export const AssetsCoin = () => {
  const account = useCurrentAccount()
  const sessionKey = useCurrentSession()
  const { toast } = useToast()

  const { mutateAsync: transferCoin } = useTransferCoin()

  const [recipient, setRecipient] = useState('')
  const [amount, setAmount] = useState('')
  const [transferLoading, setTransferLoading] = useState(false)
  const [error, setError] = useState('')

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
    address: sessionKey?.getAddress() || '',
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
    setTransferLoading(false)
    setError('')
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

  useEffect(() => {
    if (!data) {
      return
    }

    if (data.has_next_page) {
      mapPageToNextCursor.current[paginationModel.page] = data.next_cursor ?? null
    }
  }, [paginationModel, data])

  const handleAmountChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const value = event.target.value

    // Validate the input to ensure it's a valid number with at most one decimal point
    const validNumberRegex = /^[0-9]*\.?[0-9]*$/
    if (validNumberRegex.test(value)) {
      setAmount(value)

      if (selectedCoin) {
        const amountNumber = Number(value)
        const balanceNumber = Number(selectedCoin.balance) / 10 ** selectedCoin.decimals
        if (amountNumber > balanceNumber) {
          setError('Amount exceeds available balance.')
        } else {
          setError('')
        }
      }
    }
  }

  const handleRecipientChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = event.target.value
    setRecipient(value)
  }

  const handleTransferCoin = async () => {
    if (recipient === '' || amount === '0' || !selectedCoin || error) {
      setError('Please enter a valid recipient and amount.')
      return
    }

    const amountNumber = Math.floor(Number(amount) * 10 ** selectedCoin.decimals)

    setTransferLoading(true)

    try {
      await transferCoin({
        account: sessionKey!,
        recipient: recipient,
        amount: amountNumber,
        coinType: selectedCoin.coin_type,
      })
      toast({
        title: 'Transfer Successful',
        description: `Transferred ${amount} ${selectedCoin.name} to ${recipient}`,
        action: <ToastAction altText="Close">Close</ToastAction>,
      })
    } catch (error) {
      console.error('Transfer failed', error)
      toast({
        title: 'Transfer Failed',
        description: 'The transfer could not be completed. Please try again.',
        action: <ToastAction altText="Close">Close</ToastAction>,
      })
    } finally {
      setTransferLoading(false)
      handleClose()
      setError('')
    }
  }

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
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
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
                <TableCell>{formatCoin(Number(coin.balance), coin.decimals, coin.decimals)}</TableCell>
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
                      value={recipient}
                      onChange={handleRecipientChange}
                      disabled={transferLoading}
                      required
                      rows={1}
                    />
                  </div>

                  {/* Token + Balance */}
                  <div className="grid w-full max-w-md items-center gap-1.5">
                    <div className="flex items-center justify-between">
                      <Label htmlFor="amount">Amount</Label>
                      <p className="text-xs text-muted-foreground">
                        <span>Balance: </span>
                        <span className="font-semibold text-blue-600 dark:text-blue-400">
                          {selectedCoin
                            ? formatCoin(Number(selectedCoin.balance), selectedCoin.decimals)
                            : '0.0'}
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
                        id="amount"
                        className="h-10 rounded-2xl bg-gray-50 dark:bg-gray-200 text-gray-800 w-48 pr-8 mr-2 overflow-hidden border-none"
                        placeholder="0.0"
                        value={amount}
                        onChange={handleAmountChange}
                        disabled={transferLoading}
                        required
                        pattern="[0-9]*\.?[0-9]*"
                      />
                    </div>
                    {error && <p className="text-red-500 text-xs mt-1">{error}</p>}
                  </div>

                  {/* CTA */}
                  <Button
                    variant="default"
                    size="default"
                    className="w-full mt-6 font-sans gap-1"
                    onClick={handleTransferCoin}
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
