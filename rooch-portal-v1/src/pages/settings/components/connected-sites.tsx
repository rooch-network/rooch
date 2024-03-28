import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Copy, Unplug } from 'lucide-react'

const apps = [
  {
    app: 'Rooch Launchpad',
    appSite: 'rooch.platform',
    contract: 'bc1pv52fp5peuz455n6vcn5kqcnqgjxu7mvpn65lcw92pfk7q89qk9ts47eu2w',
    sessionID: 0,
    grantedAt: '2023/10/01 14:30',
    expiredAt: '2024/10/01 14:30',
  },
  {
    app: 'Gas Pet',
    appSite: 'gaspet.com',
    contract: '0x17a709022071f9f423d2f043a71d366792a889222ab3e0eb97fc54693f7e6004',
    sessionID: 1,
    grantedAt: '2024/10/01 17:42',
    expiredAt: '2025/10/01 17:42',
  },
  {
    app: 'Insforrest',
    appSite: 'insforrest.com',
    contract: '0x17a709022071f9f423d2f043a71d366792a889222ab3e0eb97fc54693f7e6004',
    sessionID: 2,
    grantedAt: '2024/10/01 17:42',
    expiredAt: '2025/10/01 17:42',
  },
  {
    app: 'FOMO2048',
    appSite: 'fomo2048.com',
    contract: '0x17a709022071f9f423d2f043a71d366792a889222ab3e0eb97fc54693f7e6004',
    sessionID: 3,
    grantedAt: '2024/10/01 17:42',
    expiredAt: '2025/10/01 17:42',
  },
]

export const ConnectedSites = () => {
  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">App</TableHead>
            <TableHead>Contract</TableHead>
            <TableHead>Session ID</TableHead>
            <TableHead>Granted at</TableHead>
            <TableHead>Expired at</TableHead>
            <TableHead className="text-center">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {apps.map((app) => (
            <TableRow key={app.sessionID}>
              <TableCell className="font-medium">{app.app}</TableCell>
              {/* 完整地址仅在较大屏幕上显示 */}
              <TableCell className="hidden md:table-cell">
                <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                  {app.contract}
                  <Button variant="ghost" size="icon" className=" w-6 h-6">
                    <Copy className="w-3 h-3" />
                  </Button>
                </span>
              </TableCell>

              {/* 缩短的地址仅在移动设备上显示 */}
              <TableCell className="md:hidden">
                <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                  {`${app.contract.substring(0, 3)}...${app.contract.substring(
                    app.contract.length - 3,
                  )}`}
                  <Button variant="ghost" size="icon" className=" w-6 h-6">
                    <Copy className="w-3 h-3" />
                  </Button>
                </span>
              </TableCell>
              <TableCell className="text-muted-foreground">{app.sessionID}</TableCell>
              <TableCell className="text-muted-foreground">{app.grantedAt}</TableCell>
              <TableCell className="text-muted-foreground">{app.expiredAt}</TableCell>
              <TableCell className="text-center">
                <Button
                  variant="link"
                  size="sm"
                  className="text-red-500 dark:text-red-400 dark:hover:text-red-300 hover:text-red-600"
                >
                  <Unplug className="h-4 w-4 mr-1" />
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
