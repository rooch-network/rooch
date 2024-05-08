// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useState } from 'react'
import { RoochSessionAccount } from '@roochnetwork/rooch-sdk'

interface SessionKeyModalProps {
  isOpen: boolean
  scopes: string[]
  onAuthorize: () => Promise<RoochSessionAccount | null>
}

export const SessionKeyModal: React.FC<SessionKeyModalProps> = ({
  isOpen,
  scopes,
  onAuthorize,
}) => {
  const [loading, setLoading] = useState(false)

  const onAuthorizeWrapper = async () => {
    setLoading(true)
    await onAuthorize()
    setLoading(false)
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-70 z-50 flex justify-center items-center">
      <div className="bg-white dark:bg-zinc-800 p-4 rounded-lg shadow-lg max-w-sm w-full">
        <h2 className="text-lg font-bold mb-4">Session Authorize</h2>
        <p className="text-sm text-muted-foreground mb-2">
          The current session dose not exist or has expired. please authorize the creation of a new
          session.
        </p>
        <div className="bg-zinc-700 p-4 rounded-lg">
          {/* SCOPE */}
          <div className="flex flex-col items-start justify-start text-gray-300 text-sm overflow-auto">
            <h3 className="text-xs mb-1 font-medium text-gray-400">Scope</h3>
            {scopes.map((item) => (
              <span key={item}>{item}</span>
            ))}
          </div>
          <Separator className="bg-muted-foreground/50 h-[0.5px] my-1.5" />
          {/* MAX INACTIVE INTERVAL */}
          <div className="flex flex-col items-start justify-start text-gray-300 text-sm">
            <h3 className="text-xs mb-1 font-medium text-gray-400">Max Inactive Interval</h3>
            <span>1200</span>
          </div>
        </div>
        <div className="flex items-center justify-end mt-4">
          <div className="flex justify-end">
            <Button variant="default" size="sm" onClick={onAuthorizeWrapper} disabled={loading}>
              {loading ? 'Createing' : 'Authorize'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
