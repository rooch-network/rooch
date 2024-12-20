// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as Dialog from '@radix-ui/react-dialog'
import clsx from 'clsx'
import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'

import { BackIcon } from '../icons/BackIcon.js'
import { Heading } from '../ui/Heading.js'
import { IconButton } from '../ui/IconButton.js'
import * as styles from './ConnectModal.css.js'
import { ConnectionStatus } from './views/ConnectionStatus.js'
import { WhatIsAWallet } from './views/WhatIsAWallet.js'
import { InstallStatus } from './views/InstallStatus.js'
import { WalletList } from './wallet-list/WalletList.js'
import { Wallet, WalletNetworkType } from '../../wellet/index.js'
import { useConnectWallet } from '../../hooks/wallet/useConnectWallet.js'
import { useCurrentNetwork, useWallets } from '../../hooks/index.js'
import { ControlledModalProps, Modal } from '../ui/Modal.js'
import { SwitchNetworkView } from '../switch-network-modal/views/SwitchNetworkView.js'
import { checkWalletNetwork } from '../util/wallet.js'

type ConnectModalView =
  | 'what-is-a-wallet'
  | 'switch-network'
  | 'connection-status'
  | 'install-status'

type ConnectModalProps = {
  /** The trigger button that opens the dialog. */
  trigger: NonNullable<ReactNode>
  onSuccess?: () => void
} & ControlledModalProps

export function ConnectModal({
  trigger,
  open,
  defaultOpen,
  onOpenChange,
  onSuccess,
}: ConnectModalProps) {
  const wallets = useWallets()
  const { mutateAsync, isError } = useConnectWallet()
  const roochNetwork = useCurrentNetwork()
  const [isModalOpen, setModalOpen] = useState(open ?? defaultOpen)
  const [currentView, setCurrentView] = useState<ConnectModalView>()
  const [targetNetwork, setTargetNetwork] = useState<WalletNetworkType>()
  const [selectedWallet, setSelectedWallet] = useState<Wallet>()
  const [walletStatus, setWalletStatus] = useState<Map<string, boolean>>(new Map())

  useEffect(() => {
    wallets.forEach(async (item) => {
      const result = await item.checkInstalled()
      setWalletStatus((prev) => {
        const newS = new Map(prev)
        newS.set(item.getName(), result)
        return newS
      })
    })
  }, [wallets, setWalletStatus])
  const resetSelection = () => {
    setSelectedWallet(undefined)
    setCurrentView(undefined)
  }

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      resetSelection()
    }
    setModalOpen(open)
    onOpenChange?.(open)
  }

  const switchNetwork = async (wallet: Wallet, target: WalletNetworkType) => {
    wallet.switchNetwork(target).then(() => {
      connectWallet(wallet)
    })
  }

  const handleSelectedWallet = async (wallet: Wallet) => {
    // OneKey is a special, not work
    // User DYOR
    if (wallet.getName() === 'OneKey') {
      connectWallet(wallet)
      return
    }

    const target = await checkWalletNetwork(wallet, roochNetwork)
    if (target) {
      setTargetNetwork(target)
      setCurrentView('switch-network')
    } else {
      connectWallet(wallet)
    }
  }

  const connectWallet = (wallet: Wallet) => {
    setCurrentView('connection-status')
    mutateAsync({ wallet }).then(() => {
      if (onSuccess) {
        onSuccess()
      }
      handleOpenChange(false)
    })
  }

  let modalContent: ReactNode | undefined
  switch (currentView) {
    case 'what-is-a-wallet':
      modalContent = <WhatIsAWallet />
      break
    case 'switch-network':
      modalContent = (
        <SwitchNetworkView
          wallet={selectedWallet!}
          targetNetWork={targetNetwork!}
          switchNetwork={switchNetwork}
          onSuccess={() => connectWallet(selectedWallet!)}
        />
      )
      break
    case 'connection-status':
      modalContent = (
        <ConnectionStatus
          selectedWallet={selectedWallet!}
          hadConnectionError={isError}
          info={
            // TODO: Better solutions are needed to deal with it
            walletStatus.get('OneKey') &&
            walletStatus.get('UniSat') &&
            selectedWallet?.getName() === 'UniSat'
              ? [
                  'If UniSat does not work!',
                  'Disable One Key Wallet, Refresh the page and try again.',
                ]
              : undefined
          }
          onRetryConnection={connectWallet}
        />
      )
      break
    case 'install-status':
      modalContent = <InstallStatus selectedWallet={selectedWallet!} />
      break
    default:
      modalContent = <WhatIsAWallet />
  }

  return (
    <Modal
      trigger={trigger}
      open={isModalOpen}
      defaultOpen={defaultOpen}
      onOpenChange={onOpenChange}
    >
      <div
        className={clsx(styles.walletListContainer, {
          [styles.walletListContainerWithViewSelected]: !!currentView,
        })}
      >
        <div className={styles.walletListContent}>
          <Dialog.Title className={styles.title} asChild>
            <Heading as="h2">Connect a Wallet</Heading>
          </Dialog.Title>
          <WalletList
            selectedWalletName={selectedWallet?.getName()}
            onSelect={(wallet) => {
              if (selectedWallet?.getName() !== wallet.getName()) {
                setSelectedWallet(wallet)
                if (walletStatus.get(wallet.getName())) {
                  handleSelectedWallet(wallet)
                } else {
                  setCurrentView('install-status')
                }
              }
            }}
          />
        </div>
        <button
          className={styles.whatIsAWalletButton}
          onClick={() => setCurrentView('what-is-a-wallet')}
          type="button"
        >
          What is a Wallet?
        </button>
      </div>
      <div
        className={clsx(styles.viewContainer, {
          [styles.selectedViewContainer]: !!currentView,
        })}
      >
        <div className={styles.backButtonContainer}>
          <IconButton type="button" aria-label="Back" onClick={() => resetSelection()}>
            <BackIcon />
          </IconButton>
        </div>
        {modalContent}
      </div>
    </Modal>
  )
}
