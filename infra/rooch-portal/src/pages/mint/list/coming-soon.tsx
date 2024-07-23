// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'

const ComingSoonTokens = [
  {
    symbol: 'HDCL',
    name: 'BTC Holder Coin (with Lock)',
    distribution: 'Self-Staking (with Lock)',
  },
  {
    symbol: 'FMC',
    name: 'Free Mint Coin',
    distribution: 'Free Mint',
  },
  {
    symbol: 'fmNFT',
    name: 'Free Mint NFT',
    distribution: 'Free Mint',
  },
  {
    symbol: 'fmSFT',
    name: 'Free Mint SFT',
    distribution: 'Free Mint',
  },
  {
    symbol: 'EBC',
    name: 'Epoch Bus Coin',
    distribution: 'Epoch Bus',
  },
  {
    symbol: 'HMC',
    name: 'Hardware Mining Coin',
    distribution: 'Hardware Mining',
  },
  {
    symbol: 'BEC',
    name: 'Burn to Earn Coin',
    distribution: 'Burn to Earn',
  },
]

export const ComingSoon = () => {

  return (
    <div className="rounded-lg border w-full">
      <SkeletonTheme baseColor="#27272A" highlightColor="#444">
        <Table>
          <TableHeader>
            <TableRow>
              {ComingSoonTokens.length > 0 ? <>
                <TableHead className="w-[150px]">Symbol</TableHead>
                <TableHead className="w-[200px]">Name</TableHead>
                <TableHead className="w-[200px]">Distribution Mechanism</TableHead>
                <TableHead>Progress</TableHead>
                <TableHead className="text-center w-[150px]" >Action</TableHead>
              </> : <TableHead className="w-full"><Skeleton/></TableHead>}
            </TableRow>
          </TableHeader>
            <TableBody>
              {
                ComingSoonTokens.length > 0 ? ComingSoonTokens.map((item) => (
                  <TableRow key={item.symbol}>
                    <TableCell className="font-medium">{item.symbol}</TableCell>
                    <TableCell className="font-medium">{item.name}</TableCell>
                    <TableCell className="font-medium">{item.distribution}</TableCell>
                    <TableCell>
                      <div className="flex items-center justify-start gap-1">
                        <Progress value={0} className="w-[60%]" />
                        <span>{0}%</span>
                      </div>
                    </TableCell>
                    <TableCell className="text-center">
                      <Button variant="link" size="sm" onClick={() => {

                      }}>
                  <span className="flex items-center justify-center">
                    Coming Soon
                  </span>
                      </Button>
                    </TableCell>
                  </TableRow>
                )): <TableRow key='loading'>
                  <TableCell className="font-medium w-full"><Skeleton/></TableCell>
                </TableRow>
              }
            </TableBody>
        </Table>
      </SkeletonTheme>
    </div>
  )
}
