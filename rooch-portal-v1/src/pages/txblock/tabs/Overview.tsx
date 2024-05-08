// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'
import { Separator } from '@/components/ui/separator'
import { Copy } from 'lucide-react'
import { LedgerTxDataView1, TransactionWithInfoView } from '@roochnetwork/rooch-sdk'

type OverviewProps = {
  txData: TransactionWithInfoView
}

// TODO Distinguish between different transactions
export const Overview: React.FC<OverviewProps> = ({ txData }) => {
  return (
    <div className="flex flex-col items-start justify-start gap-3">
      {/* Block--1 */}
      <div className="flex flex-col items-start justify-start gap-5 font-medium">
        {/* Checkpoint */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Order:</span>
          </div>
          <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
            {txData.transaction.sequence_info.tx_order}
          </span>
        </div>
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Type:</span>
          </div>
          <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
            {txData.transaction.data.type.toUpperCase()}
          </span>
        </div>

        {/* Timestamp */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Timestamp:</span>
          </div>
          <span className="text-gray-800 dark:text-gray-50 tracking-tight">
            <span>None </span>
            <span className="text-muted-foreground/65 font-normal">
              {/*(Jan 16, 2024 08:16:42 +UTC)*/}
            </span>
          </span>
        </div>
      </div>

      {/* Separator */}
      <div className="w-full">
        <Separator className="bg-accent dark:bg-accent/75" />
      </div>

      {/* Block--2 */}
      <div className="flex flex-col items-start justify-start gap-5 font-medium">
        {/* Timestamp */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Transaction Action:</span>
          </div>
          <div className="text-gray-800 dark:text-gray-50 tracking-tight flex items-center justify-start gap-1.5">
            {/* Sender */}
            {/*<div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 tracking-tight hover:underline cursor-pointer">*/}
            {/*  <span className="flex items-center justify-start gap-1 tracking-tight font-mono">*/}
            {/*    <p>*/}
            {/*      {formatAddress(*/}
            {/*        '0x9b1886b1c9e6107afbb10a4d2a01dbe318776b82021b879007631496919365cb',*/}
            {/*      )}*/}
            {/*    </p>*/}
            {/*    <Copy className="w-3 h-3 text-muted-foreground" />*/}
            {/*  </span>*/}
            {/*</div>*/}

            {/* Description */}
            <span>{(txData.transaction.data as LedgerTxDataView1).action_type.toUpperCase()}</span>
            {/*<div>*/}
            {/*  <span className="text-muted-foreground/75 dark:text-muted-foreground mr-1">*/}
            {/*    0.591993272*/}
            {/*  </span>*/}
            {/*  <span className="text-blue-500 dark:text-blue-300 hover:underline cursor-pointer">*/}
            {/*    ROOCH*/}
            {/*  </span>*/}
            {/*</div>*/}
            {/*<span className="text-muted-foreground/75 dark:text-muted-foreground">to</span>*/}

            {/* Receipients */}
            {/*<div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 tracking-tight hover:underline cursor-pointer">*/}
            {/*  <span className="flex items-center justify-start gap-1 tracking-tight font-mono">*/}
            {/*    <p>*/}
            {/*      {formatAddress(*/}
            {/*        '0x26fda2e1b4525fa4de9e576156cd184c02e4414f4d33afe3c168698911784cfa',*/}
            {/*      )}*/}
            {/*    </p>*/}
            {/*    <Copy className="w-3 h-3 text-muted-foreground" />*/}
            {/*  </span>*/}
            {/*</div>*/}
          </div>
        </div>
      </div>

      {/* Separator */}
      <div className="w-full">
        <Separator className="bg-accent dark:bg-accent/75" />
      </div>

      {/* Block--3 */}
      <div className="flex flex-col items-start justify-start gap-5 font-medium">
        {/* Sender */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Sender:</span>
          </div>
          <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">
            <div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 hover:underline cursor-pointer font-mono tracking-tight">
              <span className="flex items-center justify-start gap-1">
                <p>{(txData.transaction.data as LedgerTxDataView1).sender}</p>
                <Copy className="w-3 h-3 text-muted-foreground" />
              </span>
            </div>
          </div>
        </div>

        {/* Receipients */}
        {/*<div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">*/}
        {/*  <div className="w-36">*/}
        {/*    <span>Receipients:</span>*/}
        {/*  </div>*/}
        {/*  <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">*/}
        {/*    <div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300  hover:underline cursor-pointer font-mono tracking-tight">*/}
        {/*      <span className="flex items-center justify-start gap-1">*/}
        {/*        <p>0x26fda2e1b4525fa4de9e576156cd184c02e4414f4d33afe3c168698911784cfa</p>*/}
        {/*        <Copy className="w-3 h-3 text-muted-foreground" />*/}
        {/*      </span>*/}
        {/*    </div>*/}
        {/*  </div>*/}
        {/*</div>*/}

        {/* Status */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Status:</span>
          </div>
          <div className="flex items-center justify-start gap-2">
            <img src="/icon-success.svg" alt={txData.execution_info.status.type} />
            <span className="text-gray-800 dark:text-gray-50 tracking-tight">
              {txData.execution_info.status.type.toUpperCase()}
            </span>
          </div>
        </div>

        {/*/!* Amount *!/*/}
        {/*<div*/}
        {/*  className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">*/}
        {/*  <div className="w-36">*/}
        {/*    <span>Amount:</span>*/}
        {/*  </div>*/}
        {/*  <span className="text-gray-800 dark:text-gray-50 tracking-tight flex items-center justify-start gap-1">*/}
        {/*    <span>0.591993272</span>*/}
        {/*    <img*/}
        {/*      src="/rooch_white_logo.svg"*/}
        {/*      alt=""*/}
        {/*      className="w-4 h-4 rounded-full p-0.5 bg-gray-600 dark:bg-inherit"*/}
        {/*    />*/}
        {/*  </span>*/}
        {/*</div>*/}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Event Root:</span>
          </div>
          <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">
            <div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 hover:underline cursor-pointer font-mono tracking-tight">
              <span className="flex items-center justify-start gap-1">
                <p>{txData.execution_info.event_root}</p>
                <Copy className="w-3 h-3 text-muted-foreground" />
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Event Root:</span>
          </div>
          <div className="text-gray-800 dark:text-gray-50 flex items-center justify-start gap-1.5">
            <div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 hover:underline cursor-pointer font-mono tracking-tight">
              <span className="flex items-center justify-start gap-1">
                <p>{txData.execution_info.state_root}</p>
                <Copy className="w-3 h-3 text-muted-foreground" />
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Separator */}
      <div className="w-full">
        <Separator className="bg-accent dark:bg-accent/75" />
      </div>

      {/* Block--4 */}
      <div className="flex flex-col items-start justify-start gap-5 font-medium">
        {/* Sender */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Total Gas Fee:</span>
          </div>
          <span className="text-gray-800 dark:text-gray-50 tracking-tight flex items-center justify-start gap-1">
            <span>{txData.execution_info.gas_used}</span>
            <img
              src="/rooch_white_logo.svg"
              alt=""
              className="w-4 h-4 rounded-full p-0.5 bg-gray-600 dark:bg-inherit"
            />
          </span>
        </div>
      </div>
    </div>
  )
}
