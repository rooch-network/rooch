// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from 'react'
import { Separator } from '@/components/ui/separator'
import { Copy } from 'lucide-react'
import { TransactionWithInfoView } from '@roochnetwork/rooch-sdk'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'
import 'react-loading-skeleton/dist/skeleton.css'
import { formatTimestamp } from '@/utils/format.ts'

type OverviewProps = {
  txData?: TransactionWithInfoView
}

export const Overview: React.FC<OverviewProps> = ({ txData }) => {
  const paresL1Block = () => {
    if (txData && 'block_height' in txData?.transaction.data) {
      return {
        ...txData?.transaction.data,
      }
    }

    return null
  }

  const paresL1TX = () => {
    if (txData && 'txid' in txData?.transaction.data) {
      return {
        ...txData?.transaction.data,
      }
    }

    return null
  }

  const paresL2TX = () => {
    if (txData && 'sender' in txData?.transaction.data) {
      return {
        ...txData.transaction.data,
      }
    }

    return null
  }

  return (
    <>
      <SkeletonTheme baseColor="#27272A" highlightColor="#444">
        <div className="flex flex-col items-start justify-start gap-3">
          {/* Block--1 */}
          <div className="flex flex-col items-start justify-start gap-5 font-medium">
            {/* Checkpoint */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Order:</span>
              </div>
              {
                txData ? <span
                  className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                {txData.transaction.sequence_info.tx_order}
              </span>: <Skeleton width={150} />
              }
            </div>
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Type:</span>
              </div>
              {
                txData ? <span
                  className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                {txData.transaction.data.type.toUpperCase()}
              </span> : <Skeleton width={150} />
              }
            </div>

            {/* Timestamp */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Timestamp:</span>
              </div>
              <span className="text-gray-800 dark:text-gray-50 tracking-tight">
                {txData ? <span className="text-muted-foreground/65 font-normal">
                      {formatTimestamp(Number(txData?.transaction.sequence_info.tx_timestamp))}
                    </span> : <Skeleton width={150} />
                }
              </span>
            </div>
          </div>

          {/* Separator */}
          <div className="w-full">
            <Separator className="bg-accent dark:bg-accent/75" />
          </div>

          {/* Block--2 */}
          {
            paresL2TX() ?
              <div className="flex flex-col items-start justify-start gap-5 font-medium">
                {/* Transaction Action */}
                <div
                  className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                  <div className="w-36">
                    <span>Transaction Action:</span>
                  </div>
                  <div
                    className="text-gray-800 dark:text-gray-50 tracking-tight flex items-center justify-start gap-1.5">
                <span>
                  {txData ? paresL2TX()?.action_type.toUpperCase() : <Skeleton width={150} />}
                </span>
                  </div>
                </div>
              </div>
              :
              <div className="flex flex-col items-start justify-start gap-5 font-medium">
                {/* Transaction Action */}
                <div
                  className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                  <div className="w-36">
                    <span>{paresL1Block() ? 'Block Height:' : 'Chain ID:'}</span>
                  </div>
                  <div
                    className="text-gray-800 dark:text-gray-50 tracking-tight flex items-center justify-start gap-1.5">
                <span>
                  {txData ? paresL1Block() ? paresL1Block()?.block_height : paresL1TX()?.chain_id.toUpperCase() :
                    <Skeleton width={150} />
                  }
                </span>
                  </div>
                </div>
              </div>
          }
          {/* Separator */}
          <div className="w-full">
            <Separator className="bg-accent dark:bg-accent/75" />
          </div>

          {/* Block--3 */}
          <div className="flex flex-col items-start justify-start gap-5 font-medium">
            {/* Sender */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>{paresL2TX() ? 'Sender:' : paresL1TX() ? 'Transaction ID' : 'Block Hash'}</span>
              </div>
              <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">
                {txData ? <div
                  className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 hover:underline cursor-pointer font-mono tracking-tight">
                    <span className="flex items-center justify-start gap-1">
                      <p>{paresL2TX() ? paresL2TX()?.sender : paresL1TX() ? paresL1TX()?.txid : paresL1Block()?.block_hash}</p>
                      <Copy className="w-3 h-3 text-muted-foreground" />
                    </span>
                </div> : <Skeleton width={250} />}
              </div>
            </div>

            {/* Status */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Status:</span>
              </div>
              <div className="flex items-center justify-start gap-2">
                {txData ? <>
                  <img src="/icon-success.svg" alt={txData?.execution_info.status.type} />
                  <span className="text-gray-800 dark:text-gray-50 tracking-tight">
                      {txData?.execution_info.status.type.toUpperCase()}
                    </span>
                </> : <Skeleton width={150} />}
              </div>
            </div>

            {/* Event Root */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Event Root:</span>
              </div>
              <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">
                {txData ? <div
                  className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 hover:underline cursor-pointer font-mono tracking-tight">
                    <span className="flex items-center justify-start gap-1">
                      <p>{txData?.execution_info.event_root}</p>
                      <Copy className="w-3 h-3 text-muted-foreground" />
                    </span>
                </div> : <Skeleton width={250} />}
              </div>
            </div>

            {/* State Root */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>State Root:</span>
              </div>
              <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">
                {txData ? <div
                  className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 hover:underline cursor-pointer font-mono tracking-tight">
                    <span className="flex items-center justify-start gap-1">
                      <p>{txData?.execution_info.state_root}</p>
                      <Copy className="w-3 h-3 text-muted-foreground" />
                    </span>
                </div> : <Skeleton width={250} />}
              </div>
            </div>
          </div>

          {/* Separator */}
          <div className="w-full">
            <Separator className="bg-accent dark:bg-accent/75" />
          </div>

          {/* Block--4 */}
          <div className="flex flex-col items-start justify-start gap-5 font-medium">
            {/* Total Gas Fee */}
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Total Gas Fee:</span>
              </div>
              <span className="text-gray-800 dark:text-gray-50 tracking-tight flex items-center justify-start gap-1">
                {txData ? <>
                  <span>{txData?.execution_info.gas_used}</span>
                  <img
                    src="/rooch_white_logo.svg"
                    alt=""
                    className="w-4 h-4 rounded-full p-0.5 bg-gray-600 dark:bg-inherit"
                  />
                </> : <Skeleton width={100} />}
              </span>
            </div>
          </div>
        </div>
      </SkeletonTheme>
    </>
  )
}
