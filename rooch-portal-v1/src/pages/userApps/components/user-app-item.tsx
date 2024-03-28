import { AspectRatio } from '@/components/ui/aspect-ratio'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Send } from 'lucide-react'

interface UserAppItemProps {
  id: number
  name: string
  description: string
  profileUrl: string
  logoUrl: string
  type: string
}

export const UserAppItem = ({
  id,
  name,
  description,
  profileUrl,
  logoUrl,
  type,
}: UserAppItemProps) => {
  return (
    <Card
      key={id}
      className="h-full w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
    >
      <CardHeader className="p-4">
        <div className="flex items-center justify-start gap-x-3">
          <div>
            <Avatar>
              <AvatarImage src={profileUrl} alt={description} />
              <AvatarFallback />
            </Avatar>
          </div>
          <div>
            <div className="flex items-center justify-start gap-x-2">
              <CardTitle>{name}</CardTitle>
              <Badge
                variant="outline"
                className="rounded-full border-sky-500 text-sky-500 dark:border-sky-300 dark:text-sky-300 hover:bg-sky-500/10"
              >
                {type}
              </Badge>
            </div>
            <CardDescription>{description}</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div className="mx-4 border-none rounded-lg overflow-hidden">
          <AspectRatio
            ratio={16 / 9}
            className="flex items-center justify-center overflow-hidden cursor-pointer"
          >
            <img src={logoUrl} alt="NFT Image" className="rounded-md object-cover transition-all" />
          </AspectRatio>
        </div>
      </CardContent>
      <CardFooter className="p-4">
        <Button
          variant="default"
          size="default"
          className="w-full text-white bg-blue-500 hover:bg-blue-600 font-semibold"
        >
          <div className="flex items-center justify-center gap-x-2">
            <Send className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
            {type}
          </div>
        </Button>
      </CardFooter>
    </Card>
  )
}
