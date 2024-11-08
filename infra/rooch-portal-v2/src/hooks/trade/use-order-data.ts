import BigNumber from 'bignumber.js';
import { SuiClient, getFullnodeUrl } from '@mysten/sui.js/client';

import { NETWORK, NETWORK_PACKAGE } from 'src/config/trade';

import type { MarketObject } from './use-market-data';

const client = new SuiClient({ url: getFullnodeUrl(NETWORK) });

export interface OrderItem {
  acc: string;
  amt: string;
  inscription_id: string;
  unit_price: string;
}

export async function getListingDetail(tick: string, address: string) {
  const txb = new TransactionBlock();
  txb.moveCall({
    target: `${NETWORK_PACKAGE[NETWORK].MARKET_PACKAGE_ID}::market::listing_detail`,
    arguments: [
      txb.object(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
      txb.pure(address),
    ],
  });
  const res = await client.devInspectTransactionBlock({
    transactionBlock: txb,
    sender: '0x0000000000000000000000000000000000000000000000000000000000000000',
  });
  const fields = await client.getDynamicFields({ parentId: (res.events[0].parsedJson as any).id });
  console.log('🚀 ~ file: useOrderData.ts:19 ~ getListingDetail ~ fields:', fields);

  const ids = fields.data.map((i) => i.objectId);

  const orders = await client.multiGetObjects({
    ids,
    options: {
      showContent: true,
    },
  });
  console.log('🚀 ~ file: useOrderData.ts:27 ~ getListingDetail ~ orders:', orders);

  return orders.map((i) => {
    const { fields } = ((i.data?.content as any).fields as any).value;
    console.log('🚀 ~ file: useOrderData.ts:40 ~ getListingDetail ~ fields:', fields);
    return {
      amt: fields.amt,
      acc: fields.acc,
      objectId: fields.inscription_id,
      unitPrice: fields.unit_price,
      price: new BigNumber(fields.amt).times(fields.unit_price).toString(),
      seller: address,
    } as MarketObject;
  });
}
