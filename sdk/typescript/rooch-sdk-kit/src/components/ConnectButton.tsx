// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ButtonHTMLAttributes, ReactNode } from 'react'

import { ConnectModal } from './connect-modal/ConnectModal.js'
import { StyleMarker } from './styling/StyleMarker.js'
import { Button } from './ui/Button.js'
import { useCurrentAddress } from '../hooks/index.js'
import { AddressDropdownMenu } from './addressDropdownMenu.js'

type ConnectButtonProps = {
  connectText?: ReactNode
} & ButtonHTMLAttributes<HTMLButtonElement>

export function ConnectButton({
  connectText = 'Connect Wallet',
  ...buttonProps
}: ConnectButtonProps) {
  const address = useCurrentAddress()
  return address ? (
    <StyleMarker>
      <AddressDropdownMenu />
    </StyleMarker>
  ) : (
    <ConnectModal
      trigger={
        <StyleMarker>
          <Button variant={'outline'} {...buttonProps}>
            {connectText}
          </Button>
        </StyleMarker>
      }
    />
  )
}
