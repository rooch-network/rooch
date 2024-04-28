// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { NoData } from '@/components/no-data'
import { Card, CardHeader } from '@/components/ui/card'
import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import { hexToString } from '@/utils/format.ts'

// test address
const testAddress = ''

// TODO: 1, loading, 2 pagination, 3 目前只处理了json 铭文，其他类型还需添加ui
export const IndexedAssetsOrdi = () => {
  const account = useCurrentAccount()

  const { data: result } = useRoochClientQuery('queryInscriptions', {
    filter: {
      owner: account?.getAddress() || testAddress,
    },
  })

  return !result || result.data.length === 0 ? (
    <NoData />
  ) : (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
      {result.data.map((item) => (
        <Card
          key={item.object_id}
          className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
        >
          <CardHeader className="flex items-center justify-center">
            <h3 className="text-xl md:text-2xl">
              Inscriptions #
              {item.value.content_type === 'text/plain;charset=utf-8'
                ? hexToString(item.value.body as unknown as string)
                : item.value.content_type}
            </h3>
          </CardHeader>
          {/*<CardContent className="flex items-center justify-center text-sm md:text-base">*/}
          {/*  Amount {data.amount.toLocaleString()}*/}
          {/*</CardContent>*/}
        </Card>
      ))}
    </div>
  )
}
