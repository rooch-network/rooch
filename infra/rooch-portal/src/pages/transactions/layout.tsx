// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useTranslation } from 'react-i18next'

import { useCurrentWallet } from '@roochnetwork/rooch-sdk-kit'
import { ConnectWalletHint } from '@/components/connect-wallet-hint'
import { TransactionsList } from './list'

export const TransactionsLayout = () => {
  const { t } = useTranslation()
  const { isConnected } = useCurrentWallet()
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      <div className="flex items-center justify-between space-y-2">
        <span>
          <h1 className="text-3xl font-bold tracking-tight">{t('Transactions.title')}</h1>
          <p className="text-muted-foreground text-wrap">{t('Transactions.subTitle')}</p>
        </span>
      </div>
      {
        isConnected ? <TransactionsList />: <ConnectWalletHint/>
      }
    </div>
  )
}
