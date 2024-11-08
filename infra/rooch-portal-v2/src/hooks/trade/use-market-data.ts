import { Args, Transaction } from '@roochnetwork/rooch-sdk';

import { NETWORK, NETWORK_PACKAGE } from 'src/config/trade';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';

export interface bidItemParsedJson {
  amt: string[];
  unit_price: string[];
  bid_id: string[];
  price: string[];
  bidder: string[];
}

export interface parsedJson {
  amt: string[];
  acc: string[];
  object_id: string[];
  unit_price: string[];
  price: string[];
  seller: string[];
}

export interface OrderItem {
  quantity: string;
  order_id: string;
  unit_price: string;
  owner: string;
}

export interface MarketItem extends OrderItem {
  is_bid: false;
}

export interface BidItem extends OrderItem {
  is_bid: true;
}

export function countMaxOccurrences(sortedArray: number[]) {
  if (sortedArray.length === 0) {
    return {
      fromPrice: 0,
      start: 0,
    };
  }

  let currentCount = 1;

  for (let i = 1; i < sortedArray.length; i += 1) {
    if (sortedArray[i] === sortedArray[i - 1]) {
      currentCount += 1;
    } else {
      currentCount = 1;
    }
  }

  return {
    fromPrice: sortedArray[sortedArray.length - 1],
    start: currentCount,
  };
}

export default function useMarketData(fromPrice: number, start: number, tick: string) {
  // const account = useCurrentAccount();

  // const makeDevInspectTxb = () => {
  //   const tx = new Transaction();
  //   tx.callFunction({
  //     target: `${TESTNET_ORDERBOOK_PACKAGE}::market::floor_listing`,
  //     args: [
  //       Args.objectId(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
  //       Args.bool(false),
  //       Args.u64(fromPrice < 0 ? 0n : BigInt(fromPrice)),
  //       Args.bool(true),
  //       Args.u64(start < 0 ? 0n : BigInt(start)),
  //     ],
  //     typeArgs: [
  //       '0x3::gas_coin::RGas',
  //       '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::fixed_supply_coin::FSC',
  //     ],
  //   });
  //   return tx;
  // };

  // const { data, isLoading, refetch } = useRoochClientQuery(
  //   'executeViewFunction',
  //   {
  //     target: `${TESTNET_ORDERBOOK_PACKAGE}::market::floor_listing`,
  //     args: [
  //       Args.objectId(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
  //       Args.bool(false),
  //       Args.u64(fromPrice < 0 ? 0n : BigInt(fromPrice)),
  //       Args.bool(true),
  //       Args.u64(start < 0 ? 0n : BigInt(start)),
  //     ],
  //     typeArgs: [
  //       '0x3::gas_coin::RGas',
  //       '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::fixed_supply_coin::FSC',
  //     ],
  //   },
  //   { enabled: Boolean(account?.address) }
  // );
  // console.log('🚀 ~ file: useMarketData.ts:78 ~ useMarketData ~ data:', data);

  // const mrc20MarketItem = useMemo(() => {
  //   const marketParsedJson = data?.events[0]?.parsedJson as parsedJson;
  //   const mrc20MarketObjects: MarketObject[] = marketParsedJson?.amt.map(
  //     (item, index) =>
  //       ({
  //         amt: item,
  //         acc: marketParsedJson?.acc[index],
  //         objectId: marketParsedJson?.object_id[index],
  //         unitPrice: marketParsedJson?.unit_price[index],
  //         price: marketParsedJson?.price[index],
  //         seller: marketParsedJson?.seller[index],
  //       }) as MarketObject
  //   );
  //   return mrc20MarketObjects;
  // }, [data]);

  // const hasNextPage = useMemo(
  //   () => mrc20MarketItem?.length === 50,
  //   [mrc20MarketItem, mrc20MarketItem?.length]
  // );

  return {
    mrc20MarketObjects: [],
    hasNextPage: false,
    isLoading: false,
    refetch: () => {},
  };
}

export function makeFloorListingDevInspectTxb(fromPrice: number, start: number, tick: string) {
  const tx = new Transaction();
  tx.callFunction({
    target: `${TESTNET_ORDERBOOK_PACKAGE}::market::query_order`,
    args: [
      Args.objectId(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
      Args.bool(false),
      Args.u64(fromPrice < 0 ? 0n : BigInt(fromPrice)),
      Args.bool(true),
      Args.u64(start < 0 ? 0n : BigInt(start)),
    ],
    typeArgs: [
      '0x3::gas_coin::RGas',
      '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::fixed_supply_coin::FSC',
    ],
  });
  return tx;
}

export function makeBidListingDevInspectTxb(fromPrice: number, start: number, tick: string) {
  // const txb = new TransactionBlock();
  // txb.moveCall({
  //   target: `${NETWORK_PACKAGE[NETWORK].MARKET_PACKAGE_ID}::market::highest_bid`,
  //   arguments: [
  //     txb.object(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
  //     txb.pure(fromPrice < 0 ? 0 : fromPrice),
  //     txb.pure(start < 0 ? 0 : start),
  //   ],
  // });
  // return txb;
}

// const client = new SuiClient({ url: getFullnodeUrl(NETWORK) });

export function formatMarketData(data: any) {
  if (data.effects.status.status === 'failure') {
    return [];
  }
  // const marketParsedJson = data?.events[0]?.parsedJson as parsedJson;
  // const mrc20MarketObjects: MarketObject[] = marketParsedJson?.amt.map(
  //   (item, index) =>
  //     ({
  //       amt: item,
  //       acc: marketParsedJson?.acc[index],
  //       objectId: marketParsedJson?.object_id[index],
  //       unitPrice: marketParsedJson?.unit_price[index],
  //       price: marketParsedJson?.price[index],
  //       seller: marketParsedJson?.seller[index],
  //     }) as MarketObject
  // );
  // return mrc20MarketObjects;
  return [];
}

// export function formatBidData(data: any) {
//   if (data.effects.status.status === 'failure') {
//     return [];
//   }
//   const marketParsedJson = data?.events[0]?.parsedJson as bidItemParsedJson;
//   const mrc20MarketObjects: BidItemObject[] = marketParsedJson?.amt.map(
//     (item, index) =>
//       ({
//         amt: item,
//         bidId: marketParsedJson?.bid_id[index],
//         unitPrice: marketParsedJson?.unit_price[index],
//         price: marketParsedJson?.price[index],
//         bidder: marketParsedJson?.bidder[index],
//       }) as BidItemObject
//   );
//   return mrc20MarketObjects;
// }

// export async function getMarketList(tick: string, address?: string) {
//   if (!address) {
//     return [];
//   }

//   let fromPrice = 0;
//   let start = 0;
//   // let lastPageSize: number | undefined = undefined
//   let result: MarketObject[] = [];
//   // let count = 0
//   while (true) {
//     const txb = makeFloorListingDevInspectTxb(fromPrice, start, tick);
//     // const res = await client.devInspectTransactionBlock({ sender: address, transactionBlock: txb });
//     console.log('🚀 ~ file: useMarketData.ts:127 ~ getMarketList ~ res:', res);
//     const arr = formatMarketData(res);
//     console.log('🚀 ~ file: useMarketData.ts:151 ~ getMarketList ~ arr:', arr.length);
//     result = result.concat(arr);
//     if (arr.length < 50) {
//       break;
//     }
//     // lastPageSize = arr.length
//     const { fromPrice: countedFromPrice, start: countedStart } = countMaxOccurrences(
//       arr.map((item) => Number(item.unitPrice))
//     );
//     if (fromPrice === countedFromPrice && countedStart === 50) {
//       start += 50;
//     } else {
//       fromPrice = countedFromPrice;
//       start = countedStart;
//     }
//     // count++
//   }
//   return result;
// }

// export async function getMarketListPagination(fromPrice: number, start: number, tick: string) {
//   const result: MarketObject[] = [];
//   const tx = makeFloorListingDevInspectTxb(fromPrice, start, tick);

//   const res = await client.devInspectTransactionBlock({
//     sender: '0x0000000000000000000000000000000000000000000000000000000000000000',
//     transactionBlock: tx,
//   });
//   return formatMarketData(res);
// }

// export async function getBidListPagination(fromPrice: number, start: number, tick: string) {
//   const result: BidItemObject[] = [];
//   const txb = makeBidListingDevInspectTxb(fromPrice, start, tick);
//   const res = await client.devInspectTransactionBlock({
//     sender: '0x0000000000000000000000000000000000000000000000000000000000000000',
//     transactionBlock: txb,
//   });
//   console.log('🚀 ~ file: useMarketData.ts:211 ~ getBidListPagination ~ res:', res);
//   const bidList = formatBidData(res);
//   return bidList.filter((i) => Number(i.unitPrice) !== 0);
// }

// export async function getBidOrder(address: string, tick: string) {
//   const txb = new TransactionBlock();
//   txb.moveCall({
//     target: `${NETWORK_PACKAGE[NETWORK].MARKET_PACKAGE_ID}::market::bidder_detail`,
//     arguments: [
//       txb.object(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
//       txb.pure(address),
//     ],
//   });
//   const res = await client.devInspectTransactionBlock({ sender: address, transactionBlock: txb });
//   console.log('🚀 ~ file: useMarketData.ts:239 ~ getBidOrder ~ res:', res);

//   // @ts-ignore
//   const res2 = await client.getDynamicFields({ parentId: res.events[0].parsedJson.id });
//   // const res2 = await client.getObject({ id: res.events[0].parsedJson.id })

//   console.log('🚀 ~ file: useMarketData.ts:242 ~ getBidOrder ~ res:', res2);
// }

const u64ToValue = (u8Array: Uint8Array) =>
  BigInt(
    `0x${Array.from(u8Array.reverse())
      .map((byte) => byte.toString(16).padStart(2, '0'))
      .join('')}`
  );

// export async function getBurnObject(tick: string) {
//   const txb = new TransactionBlock();
//   txb.moveCall({
//     target: `${NETWORK_PACKAGE[NETWORK].MARKET_PACKAGE_ID}::market::burn_deatail`,
//     arguments: [txb.object(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID)],
//   });
//   const res = await client.devInspectTransactionBlock({
//     sender: '0x0000000000000000000000000000000000000000000000000000000000000000',
//     transactionBlock: txb,
//   });
//   console.log('🚀 ~ file: useMarketData.ts:239 ~ getBidOrder ~ res:', res);

//   // @ts-ignore
//   const burnData = u64ToValue(res.results[0].returnValues[0][0]);

//   // @ts-ignore
//   const costSuiData = u64ToValue(res.results[0].returnValues[1][0]);
//   console.log('🚀 ~ file: useMarketData.ts:265 ~ getBurnObject ~ burnData:', burnData, costSuiData);
//   // // @ts-ignore

//   return {
//     burnMove: new BigNumber(burnData.toString()).plus(84925).toNumber(),
//     costSui: new BigNumber(costSuiData.toString()).plus(1790000000).plus(13411575000).toNumber(),
//   };
//   // const res2 = await client.getDynamicFields({ parentId: res.events[0].parsedJson.id })
//   // const res2 = await client.getObject({ id: res.events[0].parsedJson.id })

//   // console.log('🚀 ~ file: useMarketData.ts:242 ~ getBidOrder ~ res:', res2)
// }
