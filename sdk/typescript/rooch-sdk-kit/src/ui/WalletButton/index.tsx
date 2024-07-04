// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { useState } from 'react'
import { useWallets } from '../../hooks/wallet/useWallets.js'
import { SupportChain } from '../../feature/index.js'
import { Wallet as WalletIcon } from 'lucide-react'

import { walletsMaterialMap } from '../../constants/walletsMaterialMap.js'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '../components/dialog/index.js'
import { useCurrentWallet } from '../../hooks/wallet/useCurrentWallet.js'
import { Card, CardDescription, CardHeader, CardTitle } from '../components/card/index.js'
import { capitalizeFirstLetter, formatAddress } from '../../utils/walletUtils.js'
import { useConnectWallet } from '../../hooks/wallet/useConnectWallet.js'
import { Wallet } from '../../wellet/wallet.js'
export function WalletButton() {
  const [selectWalletDialog, setSelectWalletDialog] = useState(false)
  const { isConnecting, wallet } = useCurrentWallet()
  const { mutateAsync: connectWallet } = useConnectWallet()

  const wallets = useWallets().filter((wallet) => wallet.getChain() === SupportChain.BITCOIN)
  const styleThemeDark: React.CSSProperties = {
    backgroundColor: '#27272a',
    color: 'white',
    padding: '10px',
    border: 'none',
    borderRadius: '5px',
    cursor: 'pointer',
  }
  const dialogContentStyle: React.CSSProperties = {
    maxWidth: '425px',
    overflow: 'hidden',
    border: isConnecting ? '1px solid #3f3f46' : 'none',
  }
  const dialogTitleStyle: React.CSSProperties = {
    fontSize: '1.5rem',
    textAlign: 'center',
  }
  const buttonDivStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    columnGap: '0.5rem',
  }
  const walletIconStyle: React.CSSProperties = {
    height: '1rem',
    width: '1rem',
    transform: 'rotate(0) scale(1)',
    transition: 'all 0.3s ease',
    color: 'teal',
  }
  const walletAddressStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '0.25rem',
    background: 'linear-gradient(to right, #0d9488, #9333ea, #ea580c)',
    WebkitBackgroundClip: 'text',
    color: 'transparent',
    fontWeight: '900',
    transition: 'all 0.3s ease',
  }
  const currentWalletStyle: React.CSSProperties = {
    display: 'inline-flex',
    alignItems: 'center',
    border: '1px solid #2dd4bf',
    fontSize: '0.75rem',
    fontWeight: '600',
    transition: 'color 0.3s ease, background-color 0.3s ease',
    outline: 'none',
    marginLeft: '0.5rem',
    color: '#2dd4bf',
    padding: '0 0.125rem',
    borderRadius: '0.375rem',
  }
  const handleConnectSpecificWallet = async (wallet: Wallet) => {
    try {
      await connectWallet({ wallet })

      // toast.success(`${wallet?.getName()} wallet connected`)
    } catch (error) {
      // toast.error('Connection failed')
    } finally {
      setSelectWalletDialog(false)
    }
  }
  return (
    <>
      <button style={styleThemeDark} onClick={() => setSelectWalletDialog(true)}>
        <div style={buttonDivStyle}>
          <WalletIcon style={walletIconStyle} />
          <div style={walletAddressStyle}>
            {wallet ? formatAddress(wallet.getBitcoinAddress().toStr()) : 'Connect Wallet'}
          </div>
        </div>
      </button>
      <Dialog open={selectWalletDialog} onOpenChange={setSelectWalletDialog}>
        <DialogTrigger asChild />
        <DialogContent
          style={{
            ...dialogContentStyle,
          }}
        >
          {/* {isConnecting && (
            <div className="absolute inset-0 flex items-center justify-center bg-zinc-500 bg-opacity-70 z-10">
              <LoadingSpinner />
            </div>
          )} */}
          <DialogHeader>
            <DialogTitle style={{ ...dialogTitleStyle }}>Connect Wallet</DialogTitle>
          </DialogHeader>
          {wallets.map((wallet) => (
            <Card
              key={wallet.getName()}
              onClick={() => handleConnectSpecificWallet(wallet)}
              className="bg-secondary cursor-pointer hover:border-primary/20 transition-all"
            >
              <CardHeader className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center justify-start">
                    <img
                      src={walletsMaterialMap.get(wallet.getName())!.icon}
                      alt={walletsMaterialMap.get(wallet.getName())!.description}
                      className="h-[2rem] w-[2rem] rotate-0 scale-100 mr-4 object-cover"
                    />
                    <div>
                      <CardTitle>
                        <span className="flex items-center justify-start">
                          <p>{capitalizeFirstLetter(wallet.getName())} Wallet</p>
                          {wallet?.getName() === wallet.getName() && (
                            <div style={currentWalletStyle}>Current</div>
                          )}
                        </span>
                      </CardTitle>
                      <CardDescription>Connecting using {wallet.getName()} Wallet</CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center justify-center gap-1">
                    {walletsMaterialMap.get(wallet.getName())!.types.map((type) => (
                      <img
                        key={type}
                        src={`/icon-${type}.svg`}
                        alt="Unisat Icon"
                        className="h-[1.1rem] w-[1.1rem] rotate-0 scale-100"
                      />
                    ))}
                  </div>
                </div>
              </CardHeader>
            </Card>
          ))}
        </DialogContent>
      </Dialog>
    </>
  )
}
