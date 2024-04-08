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
import { CircleDot, CircleDotDashed, Copy } from 'lucide-react'
import { formatAddress } from '../../../utils/format'

const networks = [
  {
    network: 'Bitcoin',
    address: 'bc1pr6mdxnc348lua02c32ad4uyyaw3kavjz4c8jzkh5ffvuq4ryvxhsf879j5',
    status: true,
  },
  {
    network: 'Ethereum',
    address: '0xa4Baa73f17719173Ce5f31556349c5e1D5c8BB51',
    status: false,
  },
]

export const ConnectedAccount = () => {
  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Network Status</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">Networks</TableHead>
            <TableHead>Address</TableHead>
            <TableHead className="text-center">Status</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {networks.map((network) => (
            <TableRow key={network.network}>
              <TableCell className="font-medium">{network.network}</TableCell>

              {network.network === 'Ethereum' ? (
                <>
                  <TableCell>
                    <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                      Coming soon ⌛️
                    </span>
                  </TableCell>
                  <TableCell></TableCell>
                </>
              ) : (
                <>
                  <TableCell className="hidden md:table-cell">
                    <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                      {network.address}
                      <Button variant="ghost" size="icon" className=" w-6 h-6">
                        <Copy className="w-3 h-3" />
                      </Button>
                    </span>
                  </TableCell>
                  <TableCell className="md:hidden">
                    <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                      {formatAddress(network.address)}
                      <Button variant="ghost" size="icon" className=" w-6 h-6">
                        <Copy className="w-3 h-3" />
                      </Button>
                    </span>
                  </TableCell>

                  <TableCell>
                    {network.status ? (
                      <span className="text-green-500 dark:text-green-400 flex items-center justify-center">
                        <CircleDot className="w-5 h-5 pr-1" /> active
                      </span>
                    ) : (
                      <span className="text-zinc-500 dark:text-zinc-400 flex items-center justify-center">
                        <CircleDotDashed className="w-5 h-5 pr-1" />
                        inactive
                      </span>
                    )}
                  </TableCell>
                </>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
