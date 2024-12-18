// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useEffect, useState } from 'react'
import { Modal } from '../ui/Modal.js'
import { checkWalletNetwork } from '../util/wallet.js'
import { SwitchNetworkView } from './views/SwitchNetworkView.js'
import { useCurrentAddress, useCurrentNetwork, useCurrentWallet } from '../../hooks/index.js'
import { Wallet, WalletNetworkType } from '../../wellet/index.js'

export function SwitchNetworkModal() {
  const { wallet } = useCurrentWallet()
  const roochNetwork = useCurrentNetwork()
  const [open, setOpen] = useState<boolean>(false)
  const currentAddr = useCurrentAddress()
  const [targetNetwork, setTargetNetwork] = useState<WalletNetworkType>()
  useEffect(() => {
    if (!wallet) {
      return
    }

    checkWalletNetwork(wallet, roochNetwork).then((r) => {
      setOpen(r !== undefined)
      setTargetNetwork(r)
    })
  }, [wallet, currentAddr, roochNetwork])

  const switchNetwork = async (wallet: Wallet, target: WalletNetworkType) => {
    return wallet?.switchNetwork(target)
  }
  return (
    wallet && (
      <Modal trigger={<></>} open={open} defaultOpen={false} onOpenChange={(v) => setOpen(v)}>
        <SwitchNetworkView
          wallet={wallet!}
          targetNetWork={targetNetwork!}
          switchNetwork={switchNetwork}
        />
      </Modal>
    )
  )
}
