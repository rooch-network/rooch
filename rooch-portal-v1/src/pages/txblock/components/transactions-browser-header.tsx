import { useParams } from 'react-router-dom'
import { useNavigate } from 'react-router-dom'

import { ArrowLeft, Copy } from 'lucide-react'
import { formatAddress } from '@/utils/format'
import { Button } from '@/components/ui/button'
import { HoverCard, HoverCardContent, HoverCardTrigger } from '@/components/ui/hover-card'

export const TransactionsBrowserHeader = () => {
  const navigate = useNavigate()
  const { hash: txhash } = useParams()

  return (
    <>
      <Button
        className="w-fit p-0 text-muted-foreground hover:text-muted-foreground/80 hover:no-underline"
        variant="link"
        size="sm"
        onClick={() => {
          navigate('/transactions')
        }}
      >
        <ArrowLeft className="w-4 h-4 mr-1" />
        Back to Transactions page
      </Button>
      <div className="flex items-center justify-between space-y-2">
        <HoverCard>
          <HoverCardTrigger>
            <div>
              <h1 className="text-3xl font-bold tracking-tight">Transactions Block</h1>
              <div className="flex items-center justify-start gap-1 rounded-lg bg-accent w-fit px-2 py-1 mt-2 text-sm hover:cursor-pointer">
                <p className="text-muted-foreground dark:text-white/75">{formatAddress(txhash!)}</p>
                <Copy className="text-muted-foreground w-3 h-3" />
              </div>
            </div>
          </HoverCardTrigger>
          <HoverCardContent
            className="w-fit text-xs text-muted-foreground dark:text-white/75 bg-gray-100 bg-accent p-2 rounded-lg"
            align="start"
          >
            {txhash}
          </HoverCardContent>
        </HoverCard>
      </div>
    </>
  )
}
