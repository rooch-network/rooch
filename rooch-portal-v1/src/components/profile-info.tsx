import { Avatar, AvatarFallback, AvatarImage } from './ui/avatar'
import { Button } from './ui/button'

export const ProfileInfo = () => {
  return (
    <Button
      variant="ghost"
      size="sm"
      className="h-12 w-full cursor-pointer flex items-center justify-start transition-all ease-in-out"
    >
      <div className="flex items-center justify-start gap-x-3">
        <Avatar>
          {/* TODO: add jazzicons as user profile */}
          <AvatarImage src="https://github.com/shadcn.png" alt="Rooch Network" />
          <AvatarFallback>RH</AvatarFallback>
        </Avatar>
        <div className="h-full w-full flex flex-col items-start justify-center">
          <h3 className="text-base font-semibold text-zinc-500 dark:text-zinc-300">Logic</h3>
          <p className="text-zinc-400 dark:text-zinc-500">bc1qw407...0x</p>
        </div>
      </div>
    </Button>
  )
}
