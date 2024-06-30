// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useState, useMemo, useEffect, useRef, useCallback } from 'react'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import { useToast } from '@/components/ui/use-toast'
import { isValidBitcoinAddress } from '@/utils/addressValidation'
import { RoochSessionAccount, RoochClient, BalanceInfoView } from '@roochnetwork/rooch-sdk'
import { ToastAction } from '@/components/ui/toast'

export const useAssetsCoinLogic = (sessionKey: RoochSessionAccount | null, onClose: () => void) => {
  const [error, setError] = useState<string>('')
  const [amount, setAmount] = useState<string>('')
  const [recipient, setRecipient] = useState<string>('')
  const [transferLoading, setTransferLoading] = useState<boolean>(false)
  const { toast } = useToast()
  const [selectedCoin, setSelectedCoin] = useState<BalanceInfoView | null>(null)

  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 1 })
  const mapPageToNextCursor = useRef<{ [page: number]: string | null }>({})

  const handlePageChange = useCallback(
    (selectedPage: number) => {
      if (selectedPage < 0) return
      setPaginationModel({
        page: selectedPage,
        pageSize: paginationModel.pageSize,
      })
    },
    [paginationModel],
  )

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1],
      pageSize: paginationModel.pageSize,
    }),
    [paginationModel],
  )

  const { data, isLoading, isError, refetch } = useRoochClientQuery('getBalances', {
    address: sessionKey?.getAddress() || '',
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

  useEffect(() => {
    if (data?.has_next_page) {
      mapPageToNextCursor.current[paginationModel.page] = data.next_cursor ?? null
    }
  }, [paginationModel, data])

  const handleAmountChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = event.target.value
      const validNumberRegex = /^[0-9]*\.?[0-9]*$/
      if (validNumberRegex.test(value)) {
        setAmount(value)
        if (selectedCoin) {
          const amountNumber = Number(value)
          const balanceNumber = Number(selectedCoin.balance) / 10 ** selectedCoin.decimals
          setError(
            amountNumber > balanceNumber
              ? 'Amount exceeds available balance.'
              : amountNumber <= 0
              ? 'Amount must be greater than zero.'
              : '',
          )
        }
      } else {
        setError('Please enter a valid number.')
      }
    },
    [selectedCoin],
  )

  const handleRecipientChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setRecipient(event.target.value)
  }, [])

  const handleTransferCoin = useCallback(
    async (client: RoochClient, selectedCoin: BalanceInfoView | null) => {
      setTransferLoading(true)
      try {
        if (!sessionKey || (await sessionKey.isExpired())) {
          toast({
            title: 'Session Key Expired',
            description: 'The session key has expired, please authorize a new one.',
            action: <ToastAction altText="Close">Close</ToastAction>,
          })
          return
        }

        if (recipient === '' || amount === '0' || !selectedCoin || error) {
          setError('Please enter a valid recipient and amount.')
          return
        }

        if (!isValidBitcoinAddress(recipient)) {
          setError('Please enter a valid Bitcoin address.')
          return
        }

        const amountNumber = Math.floor(Number(amount) * 10 ** selectedCoin.decimals)
        const result = await client.executeTransaction({
          address: sessionKey.getAddress(),
          authorizer: sessionKey.getAuthorizer(),
          funcId: '0x3::transfer::transfer_coin',
          args: [
            { type: 'Address', value: recipient },
            { type: 'U256', value: BigInt(amountNumber) },
          ],
          tyArgs: [{ Struct: { address: '0x3', module: 'gas_coin', name: 'GasCoin' } }],
          opts: { maxGasAmount: 50000000 },
        })

        if (result.execution_info.status.type !== 'executed') {
          toast({
            title: 'Transfer Failed',
            description: 'The transfer could not be completed. Please try again later.',
            action: <ToastAction altText="Close">Close</ToastAction>,
          })
        } else {
          await refetch()
          toast({
            title: 'Transfer Successful',
            description: `Successfully transferred ${amount} ${selectedCoin.name} to ${recipient}`,
            action: <ToastAction altText="Close">Close</ToastAction>,
          })
          onClose()
        }
      } finally {
        setTransferLoading(false)
        setError('')
      }
    },
    [amount, error, recipient, sessionKey, toast],
  )

  return {
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
    refetch,
    setSelectedCoin,
    selectedCoin,
  }
}
