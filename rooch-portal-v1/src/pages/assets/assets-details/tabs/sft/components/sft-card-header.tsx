import { Badge } from '@/components/ui/badge'

export const SftCardHeader = () => {
  return (
    <div className="flex items-center justify-between">
      <Badge variant="secondary" className="rounded-lg bg-amber-500 text-white hover:bg-amber-500">
        ROOCH
      </Badge>
      <Badge variant="default" className="cursor-pointer" onClick={() => {}}>
        #537f3a
      </Badge>
    </div>
  )
}
