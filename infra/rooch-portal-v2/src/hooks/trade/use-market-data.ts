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
