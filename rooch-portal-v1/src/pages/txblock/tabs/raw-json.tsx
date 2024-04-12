import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import { Copy } from 'lucide-react'
import { useState } from 'react'
import toast from 'react-hot-toast'

export const RawJson = () => {
  const [accessPath, setAccessPath] = useState('/object/0x1')

  const { data, isPending, error } = useRoochClientQuery('getStates', accessPath, {
    refetchOnWindowFocus: false,
    retry: false,
    enabled: !!accessPath,
  })

  const copyToClipboard = () => {
    navigator.clipboard
      .writeText(JSON.stringify(data, null, 2))
      .then(() => {
        toast('Copied to clipboard!', {
          icon: '🌟',
        })
      })
      .catch((err) => {
        console.error('Failed to copy:', err)
      })
  }

  const isDataEmpty = !data || JSON.stringify(data) === '{}' || JSON.stringify(data) === '[]'

  return (
    <div className="container p-0 dark:bg-inherit">
      <div className="flex flex-col items-start justify-start gap-3">
        <div className="flex flex-col items-start justify-start gap-5 font-medium">
          <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
            <label htmlFor="access-path-id" className="">
              Access Path:
            </label>
            <Input
              type="text"
              id="access-path-id"
              className="bg-gray-50 dark:bg-zinc-900 border border-gray-300 text-gray-900 dark:bg-inherit dark:border-gray-600 dark:text-gray-300 text-sm rounded-lg focus:ring-gray-500 focus:border-gray-500 block focus:outline-gray-500 p-2.5 w-max"
              value={accessPath}
              onChange={(e) => setAccessPath(e.target.value)}
            />
            <p className={`text-sm w-32 ${error ? 'text-red-600 dark:text-red-400' : 'h-6'}`}>
              {error ? error.toString() : ''}
            </p>
          </div>

          {isPending ? (
            <p className="dark:text-gray-300">Loading...</p>
          ) : !isDataEmpty ? (
            <div className="relative p-4 bg-gray-100 dark:bg-zinc-900 rounded-lg overflow-auto">
              <pre className="whitespace-pre-wrap text-sm text-gray-900 dark:text-gray-300 word-break break-all">
                {JSON.stringify(data, null, 2)}
              </pre>
              <Button
                variant="outline"
                size="icon"
                onClick={copyToClipboard}
                className="absolute top-2 right-2 m-2 w-8 h-8 rounded-lg"
              >
                <Copy className="w-3 h-3" />
              </Button>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
