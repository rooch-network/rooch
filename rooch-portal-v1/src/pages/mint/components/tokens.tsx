import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SftsProps } from '@/common/interface'
import { Progress } from '@/components/ui/progress'
import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { MousePointer2 } from 'lucide-react'

const sfts: SftsProps[] = [
  {
    id: 0,
    sftName: 'rBTC',
    distribution: 'Self-Staking Mint',
    totalSupply: 210000000,
  },
  {
    id: 1,
    sftName: 'rOrdi',
    distribution: 'Distribution',
    totalSupply: 210000000,
  },
  {
    id: 2,
    sftName: 'EBs',
    distribution: 'Epoch Bus',
    totalSupply: 210000000,
  },
  {
    id: 3,
    sftName: 'MAG',
    distribution: 'Mint and Get',
    totalSupply: 210000000,
  },
  {
    id: 4,
    sftName: 'rRooch',
    distribution: 'Mint and Get',
    totalSupply: 210000000,
  },
]

export const Tokens = () => {
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    const timer = setTimeout(() => setProgress(66), 500)
    return () => clearTimeout(timer)
  }, [])

  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Lorem ipsum dolor sit.</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[150px]">SFT Name</TableHead>
            <TableHead>Distribution</TableHead>
            <TableHead>Total Supply</TableHead>
            <TableHead>Progress</TableHead>
            <TableHead className="text-center">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sfts.map((sft) => (
            <TableRow key={sft.id}>
              <TableCell className="font-medium">{sft.sftName}</TableCell>
              <TableCell>{sft.distribution}</TableCell>
              <TableCell>{sft.totalSupply}</TableCell>
              <TableCell>
                <div className="flex items-center justify-start gap-1">
                  <Progress value={progress} className="w-[60%]" />
                  <span>{progress}%</span>
                </div>
              </TableCell>
              <TableCell className="text-center">
                <Button variant="link" size="sm" className="hover:no-underline">
                  <span className="flex font-semibold bg-gradient-to-r bg-clip-text from-teal-500 via-purple-500 to-orange-500 text-transparent">
                    <MousePointer2 className="w-4 h-4 mr-1 text-indigo-600" />
                    Mint
                  </span>
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
