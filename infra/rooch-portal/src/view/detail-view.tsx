// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { ReactNode } from 'react'
import { ArrowLeft } from 'lucide-react'
import { useNavigate } from 'react-router-dom'

import { useCurrentWallet } from '@roochnetwork/rooch-sdk-kit'

import { Button } from '@/components/ui/button.tsx'
import { ConnectWalletHint } from '@/components/connect-wallet-hint'

type DetailViewProps = {
  back: string,
  title: string
  children: ReactNode
}

export const DetailView: React.FC<DetailViewProps> = ({back, title, children}) => {
  const navigate = useNavigate()
  const {isConnected} = useCurrentWallet()

  console.log(isConnected)

  return (
    <div className="h-full flex-1 flex-col space-y-4 flex p-4 rounded-lg shadow-custom dark:shadow-muted">
      <Button
        className="w-fit p-0 text-muted-foreground hover:text-muted-foreground/80 hover:no-underline"
        variant="link"
        size="sm"
        onClick={() => {
          navigate(back)
        }}
      >
        <ArrowLeft className="w-4 h-4 mr-1" />
        {title}
      </Button>
      {
        isConnected ?
        children: <ConnectWalletHint/>
      }
    </div>
  )
}