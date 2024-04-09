import {
  Table,
  TableBody,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

import { Button } from '@/components/ui/button'
import { GripVerticalIcon } from 'lucide-react'

const coins = [
  {
    coin: 'ROOCH',
    balance: 288.88,
    value: '$1,146.98',
  },
  {
    coin: 'BTC',
    balance: 1.88,
    value: '$52,988.00',
  },
  {
    coin: 'USDC',
    balance: 12.88,
    value: '$0.99',
  },
  {
    coin: 'ETH',
    balance: 10.99,
    value: '$2,800.00',
  },
]

export const AssetsCoin = () => {
  return (
    <div className="rounded-lg border overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[120px]">Asset</TableHead>
            <TableHead>Balance</TableHead>
            <TableHead>Value</TableHead>
            <TableHead className="text-right">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {coins.map((coin) => (
            <TableRow key={coin.coin}>
              <TableCell className="font-medium">{coin.coin}</TableCell>
              <TableCell>{coin.balance}</TableCell>
              <TableCell>{coin.value}</TableCell>
              <TableCell className="text-right">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button size="icon" variant="ghost" className="hover:rounded-lg">
                      <GripVerticalIcon className="w-5 h-5" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent className="w-56">
                    <DropdownMenuLabel>Action</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuGroup>
                      <DropdownMenuItem onClick={() => {}}>
                        Transfer
                        <DropdownMenuShortcut>⇧⌘F</DropdownMenuShortcut>
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => {}}>
                        Swap
                        <DropdownMenuShortcut>⇧⌘S</DropdownMenuShortcut>
                      </DropdownMenuItem>
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
        <TableFooter>
          <TableRow>
            <TableCell colSpan={3}>Total</TableCell>
            <TableCell className="text-right">$25,000.00</TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    </div>
  )
}
