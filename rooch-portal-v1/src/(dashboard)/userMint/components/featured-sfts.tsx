import { useEffect, useState } from 'react'
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SftsProps } from '../common/mint-interface'
import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'

import { useNavigate } from 'react-router-dom'
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
]

export const FeaturedSfts = () => {
  const [progress, setProgress] = useState(0)
  const navigate = useNavigate()

  const handleMint = (sft: SftsProps) => {
    console.log('Minting SFT with ID:', sft.id)

    let path = '/mint/sft/'
    switch (sft.distribution) {
      case 'Self-Staking Mint':
        path += `self-staking/${sft.id}`
        break
      // other cases to be added:
      default:
        path += `${sft.id}`
    }

    navigate(path)
  }

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
                <Button
                  variant="link"
                  size="sm"
                  className="dark:text-teal-400 dark:hover:text-teal-300 text-teal-500 hover:text-teal-600 font-semibold"
                  onClick={() => handleMint(sft)}
                >
                  <MousePointer2 className="w-4 h-4 mr-1" />
                  Mint
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
