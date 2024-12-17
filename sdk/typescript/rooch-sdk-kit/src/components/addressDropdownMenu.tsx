// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as DropdownMenu from '@radix-ui/react-dropdown-menu'
import clsx from 'clsx'

import * as styles from './addressDropdownMenu.css.js'
import { ChevronIcon } from './icons/ChevronIcon.js'
import { StyleMarker } from './styling/StyleMarker.js'
import { Button } from './ui/Button.js'
import { Text } from './ui/Text.js'
import { useCurrentAddress, useRoochClient } from '../hooks/index.js'
import { SessionModal } from './session-modal/SessionModal.js'
import { useCallback, useEffect, useState } from 'react'
import { useSubscribeOnRequest } from '../provider/globalProvider.js'
import { FaucetModal } from './fauct-modal/FaucetModal.js'

export function AddressDropdownMenu() {
  const address = useCurrentAddress()
  const [sessionOpen, setSessionOpen] = useState(false)
  const [faucetOpen, setFaucetOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [progress, setProgress] = useState(0)
  const subscribeOnRequestSuccess = useSubscribeOnRequest()
  const client = useRoochClient()
  const [rgasBalance, setRgasBalance] = useState(0)

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
        setRgasBalance(result.fixedBalance)
      })
  }, [address, client])

  useEffect(() => {
    getBalance()
  }, [getBalance])

  useEffect(() => {
    const unsubscribe = subscribeOnRequestSuccess((status) => {
      console.log(status)
      if (status === 'requesting') {
        startProgress()
      } else {
        getBalance()
        setIsLoading(false)
      }
    })

    return () => {
      unsubscribe()
    }
  }, [subscribeOnRequestSuccess, address, getBalance])

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
      <SessionModal trigger={<></>} open={sessionOpen} onOpenChange={(v) => setSessionOpen(v)} />
      <FaucetModal trigger={<></>} open={faucetOpen} onOpenChange={(v) => setFaucetOpen(v)} />
      <DropdownMenu.Root modal={false}>
        <StyleMarker>
          <DropdownMenu.Trigger asChild>
            <div style={{ position: 'relative', display: 'inline-block' }}>
              <Button variant={'outline'} className={styles.connectedAddress}>
                <div className={styles.addressContainer}>
                  <Text mono>{address?.toShortStr() || ''}</Text>
                  <Text className={styles.rgasBalance}>{`RGAS: ${rgasBalance.toFixed(4)}`}</Text>
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
                  setSessionOpen(true)
                }}
              >
                <Text mono>Sessions Manager</Text>
              </DropdownMenu.Item>
              <DropdownMenu.Item
                className={clsx(styles.menuItem, styles.switchMenuItem)}
                onSelect={() => {
                  setFaucetOpen(true)
                }}
              >
                <Text mono>Faucet</Text>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </StyleMarker>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </>
  )
}
