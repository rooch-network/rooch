// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useCallback, useState } from 'react'
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import {
  useCurrentSession,
  useRemoveSession,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit'
import { AlertCircle, Check, ChevronDown, ChevronUp, Copy, Loader } from 'lucide-react'

import { formatTimestamp } from '@/utils/format.ts'

import { SessionInfoResult } from '@roochnetwork/rooch-sdk'
import { copyToClipboard } from '@/utils/copyToClipboard.ts'

interface ExpandableRowProps {
  session: SessionInfoResult
  remove: (authKey: string) => void
  loading: boolean
}

const isSessionExpired = (createTime: number, maxInactiveInterval: number) => {
  const currentTime = Date.now()
  const expirationTime = createTime + maxInactiveInterval * 1000
  return currentTime > expirationTime
}

export const ManageSessions: React.FC = () => {
  const sessionKey = useCurrentSession()
  const { mutateAsync: removeSession } = useRemoveSession()
  const {
    data: sessionKeys,
    isLoading,
    isError,
    refetch,
  } = useRoochClientQuery('querySessionKeys', {
    address: sessionKey?.getAddress() || '',
  })

  const [loading, setLoading] = useState<string | null>(null)

  const remove = useCallback(
    async (authKey: string) => {
      setLoading(authKey)
      try {
        await removeSession({
          authKey: authKey,
        })
        await refetch()
      } finally {
        setLoading(null)
      }
    },
    [removeSession, refetch],
  )

  if (isLoading || isError) {
    return (
      <div className="relative p-40">
        <div className="absolute inset-0 bg-inherit bg-opacity-50 flex justify-center items-center">
          {isLoading ? (
            <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
          ) : (
            <div className="flex flex-col items-center justify-center text-center">
              <AlertCircle className="w-12 h-12 mb-4 text-red-500" />
              <p className="text-xl text-red-500 font-semibold">Error loading data</p>
              <p className="text-sm text-muted-foreground mt-2">
                Something went wrong while fetching the data. Please try again later.
              </p>
            </div>
          )}
        </div>
      </div>
    )
  }

  if (sessionKeys && sessionKeys.data.length === 0) {
    return (
      <div className="rounded-lg border w-full flex justify-center items-center h-full p-20">
        <div className="flex flex-col items-center justify-center text-center text-xl text-muted-foreground">
          <AlertCircle className="w-12 h-12 mb-4 text-zinc-500" />
          <p className="mb-2 font-semibold">No Data</p>
          <p className="text-base text-gray-500">
            No session keys found. Please check again later.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">App</TableHead>
            <TableHead>Scope (Show Session Key Scope)</TableHead>
            <TableHead>Granted at</TableHead>
            <TableHead>Last Active at</TableHead>
            <TableHead>Expiration Interval (seconds)</TableHead>
            <TableHead className="text-center">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sessionKeys?.data.map((session) => (
            <ExpandableRow
              key={session.authenticationKey}
              session={session}
              remove={remove}
              loading={loading === session.authenticationKey}
            />
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

const ExpandableRow: React.FC<ExpandableRowProps> = ({ session, remove, loading }) => {
  const [isExpanded, setIsExpanded] = useState(false)
  const [copiedKeys, setCopiedKeys] = useState<string[]>([])

  const handleCopy = (key: string) => {
    copyToClipboard(key, (value) => {
      if (value) {
        setCopiedKeys((prev) => [...prev, key])
      } else {
        setCopiedKeys((prev) => prev.filter((item) => item !== key))
      }
    })
  }

  const expired = isSessionExpired(Number(session.createTime), session.maxInactiveInterval)

  return (
    <>
      <TableRow>
        <TableCell className="font-medium">{session.appName}</TableCell>
        <TableCell className="cursor-pointer w-64" onClick={() => setIsExpanded(!isExpanded)}>
          <div className="flex items-center justify-start gap-1 w-full">
            <span className="text-muted-foreground">
              {isExpanded ? 'Hide Session Key Scopes' : 'Show Session Key Scopes'}
            </span>
            {isExpanded ? (
              <ChevronUp className="w-4 h-4 text-muted-foreground" />
            ) : (
              <ChevronDown className="w-4 h-4 text-muted-foreground" />
            )}
          </div>
        </TableCell>
        <TableCell className="text-muted-foreground">
          {formatTimestamp(session.createTime)}
        </TableCell>
        <TableCell className="text-muted-foreground">
          {formatTimestamp(session.lastActiveTime)}
        </TableCell>
        <TableCell className="text-muted-foreground">{session.maxInactiveInterval}</TableCell>
        <TableCell className="text-center">
          {loading ? (
            <div className="flex justify-center">
              <Loader className="w-5 h-5 animate-spin" />
            </div>
          ) : (
            <Button
              variant="link"
              size="sm"
              onClick={() => remove(session.authenticationKey)}
              className={`${
                expired
                  ? 'dark:text-gray-400 dark:hover:text-gray-300 hover:text-gray-600 h-full'
                  : 'text-red-500 dark:text-red-400 dark:hover:text-red-300 hover:text-red-600 h-full'
              }`}
            >
              {expired ? 'Expired (Clear)' : 'Disconnect'}
            </Button>
          )}
        </TableCell>
      </TableRow>
      {isExpanded && (
        <TableRow>
          <TableCell colSpan={6}>
            <div className="p-4 bg-gray-100 dark:bg-gray-800 rounded-md">
              <div className="flex flex-col gap-2">
                {session.scopes.map((key: string, index: number) => (
                  <div key={index} className="flex items-center justify-between">
                    <span className="text-muted-foreground">{key}</span>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="w-6 h-6"
                      onClick={() => handleCopy(key)}
                    >
                      {copiedKeys.includes(key) ? (
                        <Check className="w-3 h-3 text-green-500" />
                      ) : (
                        <Copy className="w-3 h-3" />
                      )}
                    </Button>
                  </div>
                ))}
              </div>
            </div>
          </TableCell>
        </TableRow>
      )}
    </>
  )
}

export default ManageSessions
