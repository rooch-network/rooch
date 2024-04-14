import { BitcoinIcon, LockKeyhole } from 'lucide-react'

export const SftCardContents = () => {
  return (
    <div className="flex items-center justify-center flex-col gap-0 md:gap-3 w-full">
      <h3 className="text-3xl md:text-4xl font-semibold">1,626</h3>
      <div className="text-muted-foreground flex-col items-center justify-center text-xs md:text-sm gap-1 md:flex-row">
        <div className="flex items-center justify-center">
          <LockKeyhole className="w-3 h-3 mr-1" />
          <p>Locked ROOCH:</p>
        </div>
        <div className="flex items-center justify-center">
          <p className="dark:text-amber-300 text-amber-500 font-semibold">3.1888</p>
          <BitcoinIcon className="w-4 h-4 dark:text-amber-500 text-amber-600" />
        </div>
      </div>
    </div>
  )
}
