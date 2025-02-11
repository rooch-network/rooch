import { BalanceInfoView } from '@roochnetwork/rooch-sdk';

export type TradeCoinType = {
  amount: string;
} & BalanceInfoView;
