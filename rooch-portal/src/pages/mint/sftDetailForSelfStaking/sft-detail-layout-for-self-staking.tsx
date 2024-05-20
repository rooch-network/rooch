// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { SftDetailForSelfStaking } from './components/sft-detail-for-self-staking'

export const SftDetailLayoutForSelfStaking = () => {
  const navigate = useNavigate()

  return (
    <div className="h-full flex-1 flex-col space-y-4 flex p-4 rounded-lg shadow-custom dark:shadow-muted">
      <Button
        className="w-fit p-0 text-muted-foreground hover:text-muted-foreground/80 hover:no-underline"
        variant="link"
        size="sm"
        onClick={() => {
          navigate('/mint')
        }}
      >
        <ArrowLeft className="w-4 h-4 mr-1" />
        Back to Mint page
      </Button>
      <div>
        <div className="flex items-center justify-start">
          <div className="flex flex-col items-start justify-start">
            <div className="flex flex-row items-center justify-start gap-3">
              <h1 className="text-3xl font-bold tracking-tight">rBTC</h1>
              <Badge
                variant="outline"
                className="rounded-full border-amber-500 text-amber-500 dark:border-amber-300 dark:text-amber-300 hover:bg-amber-500/10"
              >
                In Progress
              </Badge>
            </div>
            <p className="text-muted-foreground text-sm">Self-Staking</p>
          </div>
        </div>
        {/* Mint SFT Overview */}
        <SftDetailForSelfStaking />
      </div>
    </div>
  )
}
