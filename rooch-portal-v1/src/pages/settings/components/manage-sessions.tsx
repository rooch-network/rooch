// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
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
import { useState } from 'react'
import {
  useCurrentSession,
  useRemoveSession,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit'
import { Copy, ChevronDown, ChevronUp, Check } from 'lucide-react'

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

const formatTimestamp = (timestamp: number): string => {
  const date = new Date(timestamp)
  return date.toLocaleString()
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
  const { data: sessionKeys } = useRoochClientQuery('querySessionKeys', {
    address: sessionKey?.getAddress() || '',
  })

  const remove = async (authKey: string) => {
    console.log(authKey)
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

  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">App</TableHead>
            <TableHead>Session Key ID</TableHead>
            <TableHead>Session ID</TableHead>
            <TableHead>Granted at</TableHead>
            <TableHead>Inactive Interval at</TableHead>
            <TableHead className="text-center">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sessionKeys?.data.map((session: SessionInfoResult) => (
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
        <TableCell className="cursor-pointer" onClick={() => setIsExpanded(!isExpanded)}>
          <div className="flex items-center justify-start gap-1">
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
            Disconnect
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
