// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useParams } from 'react-router-dom'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

import { TransactionsTabLayout } from './tabs/layout'
import { TransactionsBrowserHeader } from './components/transactions-browser-header'

export const TransactionDetailLayout = () => {
  const { hash } = useParams()

  const { data: result } = useRoochClientQuery('queryTransactions', {
    filter: {
      tx_hashes: [hash || ''],
    },
  })

  console.log(result)

  return (
    <div className="h-full flex-1 flex-col space-y-4 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      <TransactionsBrowserHeader />
      <TransactionsTabLayout data={result?.data[0]}/>
    </div>
  )
}
