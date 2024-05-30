// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useState } from 'react'
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
import { Copy, ChevronDown, ChevronUp, Check, AlertCircle } from 'lucide-react'

import { formatTimestamp } from '@/utils/format.ts'

interface Session {
  authenticationKey: string
  appName: string
  createTime: string
  lastActiveTime: string
  maxInactiveInterval: string
  scopes: string[]
}

interface SessionInfoResult {
  authenticationKey: string
  appName: string
  createTime: number
  lastActiveTime: number
  maxInactiveInterval: number
  scopes: string[]
}

interface ExpandableRowProps {
  session: Session
  remove: (authKey: string) => void
}

const copyToClipboard = async (text: string, setCopied: (value: boolean) => void) => {
  try {
    await navigator.clipboard.writeText(text)
    setCopied(true)
    console.log('Copied to clipboard:', text)
    setTimeout(() => setCopied(false), 2000) // Reset after 2 seconds
  } catch (err) {
    console.error('Could not copy text:', err)
  }
}

export const ManageSessions: React.FC = () => {
  const sessionKey = useCurrentSession()
  const { mutateAsync: removeSession } = useRemoveSession()
  const {
    data: sessionKeys,
    isLoading,
    isError,
  } = useRoochClientQuery('querySessionKeys', {
    address: sessionKey?.getAddress() || '',
  })

  const remove = async (authKey: string) => {
    // console.log(authKey)
    await removeSession({
      authKey: authKey,
    })
  }

  const formatSession = (session: SessionInfoResult): Session => ({
    ...session,
    createTime: formatTimestamp(session.createTime),
    lastActiveTime: formatTimestamp(session.lastActiveTime),
    maxInactiveInterval: session.maxInactiveInterval.toString(),
  })

  if (!sessionKeys?.data.length) {
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
          {sessionKeys.data.map((session: SessionInfoResult) => (
            <ExpandableRow
              key={session.authenticationKey}
              session={formatSession(session)}
              remove={remove}
            />
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

const ExpandableRow: React.FC<ExpandableRowProps> = ({ session, remove }) => {
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

  return (
    <>
      <TableRow>
        <TableCell className="font-medium">{session.appName}</TableCell>
        <TableCell className="cursor-pointer w-64" onClick={() => setIsExpanded(!isExpanded)}>
          <div className="flex items-center justify-start gap-1 w-full">
            <span className="text-muted-foreground">
              {isExpanded ? 'Hide Session Keys' : 'Show Session Keys'}
            </span>
            {isExpanded ? (
              <ChevronUp className="w-4 h-4 text-muted-foreground" />
            ) : (
              <ChevronDown className="w-4 h-4 text-muted-foreground" />
            )}
          </div>
        </TableCell>
        <TableCell className="text-muted-foreground">{session.createTime}</TableCell>
        <TableCell className="text-muted-foreground">{session.lastActiveTime}</TableCell>
        <TableCell className="text-muted-foreground">{session.maxInactiveInterval}</TableCell>
        <TableCell className="text-center">
          <Button
            variant="link"
            size="sm"
            onClick={() => remove(session.authenticationKey)}
            className="text-red-500 dark:text-red-400 dark:hover:text-red-300 hover:text-red-600"
          >
            Expired (Clear)
          </Button>
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
