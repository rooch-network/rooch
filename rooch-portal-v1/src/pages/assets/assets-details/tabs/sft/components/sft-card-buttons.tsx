import { Button } from '@/components/ui/button'

export const SftCardButtons = () => {
  return (
    <>
      <Button
        size="sm"
        variant="outline"
        className="rounded-lg cursor-pointer bg-inherit text-teal-500 border-teal-500 dark:hover:border-teal-400 dark:hover:text-teal-400 hover:bg-teal-500/15 w-full flex-1 hover:border-teal-600 hover:text-teal-600"
      >
        Split
      </Button>
      <Button
        size="sm"
        variant="outline"
        className="rounded-lg cursor-pointer bg-inherit border-red-500 text-red-500 dark:hover:border-red-400 dark:hover:text-red-400 hover:bg-red-500/15 w-full flex-1 hover:border-red-600 hover:text-red-600"
      >
        Burn
      </Button>
      <Button
        size="sm"
        variant="outline"
        className="rounded-lg cursor-pointer bg-inherit border-amber-500 text-amber-500 dark:hover:border-amber-400 dark:hover:text-amber-400 hover:bg-amber-500/15 w-full flex-1 hover:border-amber-600 hover:text-amber-600"
      >
        Transfer
      </Button>
    </>
  )
}
