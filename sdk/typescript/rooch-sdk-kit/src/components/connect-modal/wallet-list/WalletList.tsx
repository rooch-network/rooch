// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useWallets } from '../../../hooks/wallet/useWallets.js'
import * as styles from './WalletList.css.js'
import { WalletListItem } from './WalletListItem.js'
import { Wallet } from '../../../wellet/wallet.js'

type WalletListProps = {
  selectedWalletName?: string
  onSelect: (wallet: Wallet) => void
}

export function WalletList({ selectedWalletName, onSelect }: WalletListProps) {
  const wallets = useWallets()

  return (
    <ul className={styles.container}>
      {wallets.map((wallet) => (
        <WalletListItem
          key={wallet.getName()}
          name={wallet.getName()}
          icon={wallet.getIcon()}
          isSelected={wallet.getName() === selectedWalletName}
          onClick={() => onSelect(wallet)}
        />
      ))}
    </ul>
  )
}
