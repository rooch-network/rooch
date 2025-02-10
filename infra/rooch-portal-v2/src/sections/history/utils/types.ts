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
