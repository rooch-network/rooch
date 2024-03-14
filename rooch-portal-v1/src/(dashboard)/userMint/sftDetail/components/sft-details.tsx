import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { useEffect, useState } from 'react'

export function SftDetails() {
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    const timer = setTimeout(() => setProgress(84.4), 500)
    return () => clearTimeout(timer)
  }, [])

  return (
    <>
      <div className="flex items-center justify-start w-full gap-2 text-muted-foreground dark:text-teal-50 mt-2">
        <span className="text-sm">Process</span>
        <Progress value={progress} />
        <span className="text-sm flex items-center gap-1">
          <p className="font-semibold">84.4%</p>
          <p>Minted</p>
        </span>
      </div>
      <div className="grid md:grid-cols-2 gap-x-6 w-full mt-6 gap-4 md:gap-6">
        <div className="w-full h-full">
          <Card className="border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
            <CardHeader className="dark:text-teal-100">
              <CardTitle>Overview</CardTitle>
              <CardDescription className="dark:text-teal-50/70">
                Deploy your new project in one-click.
              </CardDescription>
            </CardHeader>
            <CardContent className="text-sm dark:text-primary grid md:grid-cols-2 gap-6 w-full px-6 md:px-0">
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
            </CardContent>
          </Card>
        </div>
        <div className="w-full h-full">
          <Card className="border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
            <CardHeader className="dark:text-teal-100">
              <CardTitle>Progress</CardTitle>
              <CardDescription className="dark:text-teal-50/70">
                Deploy your new project in one-click.
              </CardDescription>
            </CardHeader>
            <CardContent className="text-sm dark:text-primary grid md:grid-cols-2 gap-6 w-full px-6 md:px-0">
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
              <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-teal-50">
                <span className="text-4xl font-medium md:text-5xl">12,333</span>
                <span className="text-base md:text-sm">Distribution Type</span>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
      <Button className="rounded-lg w-full mt-4 mb-2 md:mt-8 md:mb-6 h-12 dark:bg-teal-500 dark:hover:bg-teal-400 text-white bg-teal-500 hover:bg-teal-600 font-semibold">
        Mint
      </Button>
    </>
  )
}
