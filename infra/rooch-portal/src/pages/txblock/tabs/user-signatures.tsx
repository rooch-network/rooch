// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useState, useEffect } from 'react'
import { Copy } from 'lucide-react'
import { TransactionSequenceInfoView } from '@roochnetwork/rooch-sdk'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'
import 'react-loading-skeleton/dist/skeleton.css'

type UserSignaturesProps = {
  seqData: TransactionSequenceInfoView | null
}

enum Auth {
  ED25519,
  ECDSA_K1,
}

export const UserSignatures: React.FC<UserSignaturesProps> = ({ seqData }) => {
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (seqData) {
      setLoading(false)
    }
  }, [seqData])

  const validatorId = seqData ? parseInt(seqData.tx_order_signature.auth_validator_id) : 0

  return (
    <SkeletonTheme baseColor="#27272A" highlightColor="#444">
      <div className="flex flex-col items-start justify-start gap-3">
        <div className="flex flex-col items-start justify-start gap-5 font-medium">
          {/* Schema */}
          <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
            <div className="w-36">
              <span>Schema:</span>
            </div>
            <span className="text-gray-800 dark:text-gray-50 tracking-tight">
              {loading ? (
                <Skeleton width={100} />
              ) : (
                <span>{Auth[validatorId > 1 ? validatorId - 1 : validatorId]}</span>
              )}
            </span>
          </div>

          {/* Payload */}
          <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
            <div className="w-36">
              <span>Payload:</span>
            </div>
            <div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 tracking-tight hover:underline cursor-pointer">
              <span className="flex items-center justify-start gap-1 tracking-tight font-mono">
                {loading || !seqData ? (
                  <Skeleton width={300} />
                ) : (
                  <>
                    <p>{seqData.tx_order_signature.payload}</p>
                    <Copy className="w-3 h-3 text-muted-foreground" />
                  </>
                )}
              </span>
            </div>
          </div>

          {/* Signature (commented out, can be added similarly) */}
          {/* <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
            <div className="w-36">
              <span>Signature:</span>
            </div>
            <span className="text-gray-800 dark:text-gray-50 tracking-tight">
              {loading ? <Skeleton width={300} /> : <span>{seqData.tx_order_signature.signature}</span>}
            </span>
          </div> */}
        </div>
      </div>
    </SkeletonTheme>
  )
}
