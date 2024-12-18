// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import clsx from 'clsx'
import { useCallback, useEffect, useState } from 'react'
import * as DropdownMenu from '@radix-ui/react-dropdown-menu'

import * as styles from './DropdownMenu.css.js'

import { Text } from './ui/Text.js'
import { Button } from './ui/Button.js'
import { ChevronIcon } from './icons/ChevronIcon.js'
import { StyleMarker } from './styling/StyleMarker.js'
import { SessionModal } from './session-modal/SessionModal.js'
import { FaucetModal } from './fauct-modal/FaucetModal.js'
import { SwapGasModal } from './swap-gas-modal/SwapGasModal.js'
import { SwitchNetworkModal } from './switch-network-modal/SwitchNetworkModal.js'

import { useCurrentAddress, useRoochClient } from '../hooks/index.js'
import { useSubscribeOnError, useSubscribeOnRequest } from '../provider/globalProvider.js'
import { ErrorValidateCantPayGasDeposit } from '@roochnetwork/rooch-sdk'

export function ActionDropdownMenu() {
  const address = useCurrentAddress()
  const [sessionOpen, setSessionOpen] = useState(false)
  const [faucetOpen, setFaucetOpen] = useState(false)
  const [swapGasOpen, setSwapGasOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [progress, setProgress] = useState(0)
  const subscribeOnRequest = useSubscribeOnRequest()
  const subscribeOnError = useSubscribeOnError()
  const client = useRoochClient()
  const [rGasBalance, setRGasBalance] = useState(0)

  const getBalance = useCallback(() => {
    if (!address) {
      return
    }
    client
      .getBalance({
        owner: address,
        coinType: '0x3::gas_coin::RGas',
      })
      .then((result) => {
        setRGasBalance(result.fixedBalance)
      })
  }, [address, client])

  useEffect(() => {
    getBalance()
  }, [getBalance])

  useEffect(() => {
    const unsubscribe = subscribeOnRequest((status) => {
      switch (status) {
        case 'requesting':
          startProgress()
          break
        case 'error':
          setIsLoading(false)
          break
        case 'success':
          getBalance()
          setIsLoading(false)
          break
      }
    })

    const UnsubscribeOnError = subscribeOnError((error) => {
      if (error.code === ErrorValidateCantPayGasDeposit) {
        setFaucetOpen(true)
      }
    })

    return () => {
      unsubscribe()
      UnsubscribeOnError()
    }
  }, [subscribeOnRequest, subscribeOnError, address, getBalance])

  const startProgress = () => {
    setIsLoading(true)
    setProgress(0)
    const interval = setInterval(() => {
      setProgress((prev) => {
        const nextProgress = prev + 10
        if (nextProgress >= 90) {
          clearInterval(interval)
        }
        return nextProgress
      })
    }, 100)
  }
  return (
    <>
      <SwitchNetworkModal />
      <SessionModal trigger={<></>} open={sessionOpen} onOpenChange={(v) => setSessionOpen(v)} />
      <FaucetModal
        trigger={<></>}
        open={faucetOpen}
        onOpenChange={(v) => setFaucetOpen(v)}
        swapRGas={() => {
          setFaucetOpen(false)
          setSwapGasOpen(true)
        }}
      />
      <SwapGasModal trigger={<></>} open={swapGasOpen} onOpenChange={(v) => setSwapGasOpen(v)} />
      <DropdownMenu.Root modal={false}>
        <StyleMarker>
          <DropdownMenu.Trigger asChild>
            <div style={{ position: 'relative', display: 'inline-block' }}>
              <Button variant={'outline'} className={styles.connectedAddress}>
                <div className={styles.addressContainer}>
                  <Text mono>{address?.toShortStr() || ''}</Text>
                  <Text className={styles.rgasBalance}>{`RGAS: ${rGasBalance.toFixed(4)}`}</Text>
                </div>
                <ChevronIcon />
                {isLoading && (
                  <div className={styles.progressBar} style={{ width: `${progress}%` }} />
                )}
              </Button>
            </div>
          </DropdownMenu.Trigger>
        </StyleMarker>
        <DropdownMenu.Portal>
          <StyleMarker className={styles.menuContainer}>
            <DropdownMenu.Content className={styles.menuContent}>
              <DropdownMenu.Item
                className={clsx(styles.menuItem, styles.switchMenuItem)}
                onSelect={() => {
                  setFaucetOpen(true)
                }}
              >
                <Text mono>Faucet</Text>
              </DropdownMenu.Item>
              <DropdownMenu.Item
                className={clsx(styles.menuItem, styles.switchMenuItem)}
                onSelect={() => {
                  setSwapGasOpen(true)
                }}
              >
                <Text mono>Swap RGas</Text>
              </DropdownMenu.Item>
              <DropdownMenu.Item
                className={clsx(styles.menuItem, styles.switchMenuItem)}
                onSelect={() => {
                  setSessionOpen(true)
                }}
              >
                <Text mono>Sessions Manager</Text>
              </DropdownMenu.Item>
              <DropdownMenu.Item
                className={clsx(styles.menuItem, styles.switchMenuItem)}
                onSelect={() => {
                  window.localStorage.clear()
                  window.location.reload()
                }}
              >
                <Text mono>Disconnect</Text>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </StyleMarker>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </>
  )
}
