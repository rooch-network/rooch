import { useSuiClient } from "@mysten/dapp-kit";
import { MarketEvent, Inscription, RenderEvent } from "src/sections/history/utils/types";
import { nanoid } from 'nanoid'
import { EVENT_ENUM, EVENT_MAP, parseEvent } from "src/sections/history/utils/common";
import BigNumber from "bignumber.js";
import { EventId } from "@mysten/sui.js/client";
import { useQuery } from "@tanstack/react-query";
import { NETWORK, NETWORK_PACKAGE } from "src/config/constant";

export default function useTransactionEvent() {
  const client = useSuiClient()

  const getTransactionEvent = async (cursor?: EventId | null) => {
    return await client.queryEvents({
      query: {
        MoveModule: {
          package: '0xb88f9149c55d6314b7ae436a730cfbf3a5d53522f37e2d56288531bb8adc7071',
          module: 'market',
        },
      },
      cursor,
      // order: 'descending',
      limit: 50,
    })
  }

  const getInscriptionInfo = async (ids: string[]) => {
    return await client.multiGetObjects({
      ids,
      options: {
        showContent: true
      }
    })
  }

  const fetch = async () => {
    let count = 0
    let res: MarketEvent['data'] = []
    let cursor: EventId | null = null
    while (true) {
      const transactionEvent = await getTransactionEvent(cursor) as MarketEvent
      res.push(...transactionEvent.data)
      cursor = transactionEvent.nextCursor
      if (count === 3) {
        break;
      }
      count++
    }
    console.log('🚀 ~ file: useTransactionEvent.ts:48 ~ fetch ~ res:', res)
    return res.filter(i => parseEvent(i.type)).map((eventItem) => {
      const type = EVENT_MAP[parseEvent(eventItem.type)!]
      return {
        id: nanoid(),
        type,
        digest: eventItem.id.txDigest,
        timestampMs: eventItem.timestampMs,
        operator: eventItem.parsedJson.operator,
        tick: eventItem.parsedJson.tick || '',
        unitPrice: type === EVENT_ENUM.ListedEvent ? eventItem.parsedJson.price : eventItem.parsedJson.unit_price || eventItem.parsedJson.per_price || '0',
        amount: (type === EVENT_ENUM.BurnFloorEvent ? eventItem.parsedJson.amt : eventItem.parsedJson.inscription_amount) || eventItem.parsedJson.amt || '0',
        costSui: (type === EVENT_ENUM.BurnFloorEvent ? eventItem.parsedJson.cost_sui : '0') || '0',
        price: type === EVENT_ENUM.ListedEvent ? new BigNumber(eventItem.parsedJson.price).times(eventItem.parsedJson.inscription_amount).toString() : eventItem.parsedJson.price,
        bidder: eventItem.parsedJson.bidder || '',
        buyer: eventItem.parsedJson.to || '',
        seller: eventItem.parsedJson.from || '',
        sender: eventItem.sender,
        // inscription: find ? (find as any).fields as Inscription : undefined,
      };
    })
  }

  const { data, isLoading, refetch, isFetching } = useQuery<RenderEvent[]>({
    queryKey: ['transaction-events'], queryFn: async () => {
      return await fetch()
    },
  })
  return { transactionEvent: data, isLoading, isFetching, refetch }
}