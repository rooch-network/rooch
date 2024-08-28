export interface RedEnvelopeItem {
  claim_type: 0 | 1;
  claimed_address: string[];
  coin_store: CoinStore;
  end_time: string;
  sender: string;
  start_time: string;
  total_coin: string;
  total_envelope: string;
}

export interface CoinStore {
  abilities: number;
  type: string;
  value: Value;
}

export interface Value {
  id: string;
}
