// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { useState, useEffect, useRef, useMemo } from 'react'
import parse from 'html-react-parser'
import { useCurrentWallet, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

import { useTranslation } from 'react-i18next'
import { CursorType } from '@/common/interface'
import { UtxoCard } from '@/pages/mint/detail/components/utxo-card.tsx'
import CustomPagination from '@/components/custom-pagination.tsx'

type UTXOViewProps = {
  owner: string
  selectedUTXOCallback?: (id: string) => void
}

export const UtxoView: React.FC<UTXOViewProps> = ({ owner, selectedUTXOCallback }) => {
  const { t } = useTranslation()
  const { wallet } = useCurrentWallet()
  const [selectedUTXO, setSelectUTXO] = useState('')
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })
  const mapPageToNextCursor = useRef<{ [page: number]: CursorType }>({})

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1] || undefined,
      pageSize: paginationModel.pageSize.toString(),
    }),
    [paginationModel],
  )

  const { data: utxos, isPending: utxosIsPending } = useRoochClientQuery('queryUTXO', {
    filter: {
      owner: owner,
    },
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) return

    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    })
  }

  const toggleUTXOSelected = (utxoId: string) => {
    if (!selectedUTXOCallback) {
      return
    }

    if (utxoId !== selectedUTXO) {
      setSelectUTXO(utxoId)
      selectedUTXOCallback(utxoId)
    }
  }

  useEffect(() => {
    if (!utxos) {
      return
    }

    if (utxos) {
      mapPageToNextCursor.current[paginationModel.page] = utxos.next_cursor ?? null
    }
  }, [paginationModel, utxos])

  return utxosIsPending ? (
    <UtxoCard
      key="loading-utxo"
      utxo={undefined}
      selected={false}
      selectedCallback={toggleUTXOSelected}
    />
  ) : utxos && utxos.data.length > 0 ? (
    <>
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {utxos.data.map((utxo) => (
          <UtxoCard
            key={utxo.tx_order}
            utxo={utxo}
            selected={utxo.id === selectedUTXO}
            selectedCallback={toggleUTXOSelected}
          />
        ))}
      </div>
      <CustomPagination
        currentPage={paginationModel.page}
        hasNextPage={utxos.has_next_page}
        onPageChange={handlePageChange}
      />
    </>
  ) : (
    <>
      {parse(
        t('UTXO.null')
          .replace(/\n/g, '<br />')
          .replace(
            'Bitcoin explorer',
            `<a
          href="https://mempool.space/testnet/address/${wallet?.getBitcoinAddress()?.toStr()}"
          target="_blank"
          className="inline font-medium text-blue-600 underline dark:text-blue-500 underline-offset-2 decoration-600 dark:decoration-500 decoration-solid hover:text-blue-500 dark:hover:text-blue-400 transition-all"
          rel="noreferrer"
      >
        Bitcoin Explorer
      </a>`,
          ),
      )}
    </>
  )
}
