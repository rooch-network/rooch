// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
import { Separator } from '@/components/ui/separator.tsx'

export interface AppItemProps {
  id: number
  name: string
  description: string
  profileUrl: string
  logoUrl: string
  type: string
  url: string
}

export const AppsItem = ({ id, name, description, profileUrl, logoUrl, type, url }: AppItemProps) => {
  return (
    <Card
      key={id}
      className="h-full w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
    >
      <CardHeader className="p-4 pb-2">
        <div className="flex items-center justify-start gap-x-3">
          <div>
            <Avatar className='bg-white'>
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
            <CardDescription className='mt-1'>{description}</CardDescription>
          </div>
        </div>
      </CardHeader>
      <div className="w-full">
        <Separator className="bg-accent dark:bg-accent/75" />
      </div>
      <a href={url} className="w-full" target="_blank" rel="noopener noreferrer">
      <CardContent className="p-0">
        <div className="mx-4 border-none rounded-lg overflow-hidden">
          <AspectRatio
            ratio={16 / 9}
            className="flex items-center justify-center overflow-hidden cursor-pointer"
          >
            <img src={logoUrl} alt="Website" className="rounded-md object-cover transition-all" />

          </AspectRatio>
        </div>
      </CardContent>
      </a>
      <CardFooter className="p-4">
        <a href={url} className="w-full" target="_blank" rel="noopener noreferrer">
        <Button variant="default" size="default" className="w-full">
          <div className="flex items-center justify-center gap-x-2">
            {type}
          </div>
        </Button>
        </a>
      </CardFooter>
    </Card>
  )
}
