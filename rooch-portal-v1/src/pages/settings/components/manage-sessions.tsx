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
import { Copy } from 'lucide-react'

// TODO: 1. fetch/remove loading, 2. The first boot creates a session and introduces a session
export const ManageSessions = () => {
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
          {sessionKeys?.data.map((session) => (
            <TableRow key={session.authenticationKey}>
              <TableCell className="font-medium">{session.appName}</TableCell>
              {/* 完整地址仅在较大屏幕上显示 */}
              <TableCell className="hidden md:table-cell">
                <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                  {session.scopes}
                  <Button variant="ghost" size="icon" className=" w-6 h-6">
                    <Copy className="w-3 h-3" />
                  </Button>
                </span>
              </TableCell>

              {/* 缩短的地址仅在移动设备上显示 */}
              {/*<TableCell className="md:hidden">*/}
              {/*  <span className="flex items-center justify-start gap-0.5 text-muted-foreground">*/}
              {/*    {`${session.contract.substring(0, 3)}...${app.contract.substring(*/}
              {/*      app.contract.length - 3,*/}
              {/*    )}`}*/}
              {/*    <Button variant="ghost" size="icon" className=" w-6 h-6">*/}
              {/*      <Copy className="w-3 h-3" />*/}
              {/*    </Button>*/}
              {/*  </span>*/}
              {/*</TableCell>*/}
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
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

export default ManageSessions
