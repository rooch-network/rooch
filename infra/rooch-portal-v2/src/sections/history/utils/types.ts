export interface MarketEvent {
  data: {
    id: {
      txDigest: string;
      eventSeq: string;
    };
    packageId: string;
    transactionModule: string;
    sender: string;
    type: string;
    parsedJson: {
      id: string;
      tick: string;
      inscription_amount: string;
      operator: string;
      price: string;
      from?: string;
      to?: string;
      unit_price?: string;
      per_price?: string;
      amt?: string;
      bidder?: string;
      cost_sui?: string;
    };
    bcs: string;
    timestampMs: string;
  }[];
  hasNextPage: boolean;
  nextCursor: {
    eventSeq: string;
    txDigest: string;
  };
}

export interface Inscription {
  acc: string;
  amount: string;
  attach_coin: string;
  id: {
    id: string;
  };
  metadata: string | null;
  tick: string;
}

export interface RenderEvent {
  order_id: string;
  order_type: 0 | 1 | 2 | 3 | 4 | 5;
  owner: string;
  quantity: string;
  timestamp: string;
  unit_price: string;
  created_at: string;
  sender: string;
  tx_hash: string;
}
