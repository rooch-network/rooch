import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import type { BidItem } from 'src/hooks/trade/use-market-data';

import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { Card, Stack, Button, CardActions } from '@mui/material';

import { fromDust } from 'src/utils/number';
import { formatUnitPrice } from 'src/utils/marketplace';

import { toast } from 'src/components/snackbar';

import OrderShopCard from './order-shop-card';
import { useNetworkVariable } from '../../hooks/use-networks';

export type OrderItemBidCardProps = {
  item: BidItem;
  tick: string;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  onAcceptBid: (item: BidItem) => void;
  onRefetchMarketData: () => Promise<void>;
};

export default function OrderItemBidCard({
  item,
  tick,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  onAcceptBid,
  onRefetchMarketData,
}: OrderItemBidCardProps) {
  const market = useNetworkVariable('market');
  const account = useCurrentAddress();
  const { mutate: signAndExecuteTransaction, isPending } = useSignAndExecuteTransaction();

  const price = useMemo(
    () =>
      new BigNumber(formatUnitPrice(item.unit_price, toCoinBalanceInfo.decimals))
        .times(fromDust(item.quantity, toCoinBalanceInfo.decimals))
        .toString(),
    [toCoinBalanceInfo.decimals, item.quantity, item.unit_price]
  );

  return (
    <Card
      key={`${item.order_id}-${item.owner}-bid`}
      sx={{
        '&:hover .add-cart-btn': {
          opacity: 1,
        },
        p: 1,
      }}
      onClick={() => {}}
    >
      <OrderShopCard
        fromCoinBalanceInfo={fromCoinBalanceInfo}
        toCoinBalanceInfo={toCoinBalanceInfo}
        objectId={item.order_id}
        tick={tick}
        isVerified
        amount={item.quantity}
        seller={item.owner}
        price={price}
        unitPrice={formatUnitPrice(item.unit_price, toCoinBalanceInfo.decimals)}
        selectMode={false}
        type="bid"
      />
      <CardActions>
        <Stack
          direction="row"
          sx={{
            width: '100%',
          }}
          justifyContent="space-around"
          spacing={2}
        >
          {account?.genRoochAddress().toHexAddress() === item.owner ? (
            <LoadingButton
              loading={isPending}
              variant="outlined"
              color="warning"
              fullWidth
              size="small"
              onClick={() => {
                const tx = new Transaction();
                tx.callFunction({
                  target: `${market.orderBookAddress}::market_v2::cancel_order`,
                  args: [
                    Args.objectId(market.tickInfo[tick.toLowerCase()].obj),
                    Args.u64(BigInt(item.order_id)),
                  ],
                  typeArgs: [fromCoinBalanceInfo.coin_type, toCoinBalanceInfo.coin_type],
                });
                signAndExecuteTransaction(
                  {
                    transaction: tx,
                  },
                  {
                    async onSuccess(data) {
                      if (data.execution_info.status.type === 'executed') {
                        toast.success('Cancel Bid Success');
                        onRefetchMarketData();
                      } else {
                        toast.error('Cancel Bid Failed');
                      }
                    },
                    onError(error) {
                      toast.error(String(error));
                    },
                  }
                );
              }}
            >
              Cancel Bid
            </LoadingButton>
          ) : (
            <Button
              variant="outlined"
              color="error"
              fullWidth
              size="small"
              disabled={
                Boolean(!account?.genRoochAddress().toHexAddress()) ||
                new BigNumber(toCoinBalanceInfo.balance || 0).isLessThan(item.quantity)
              }
              onClick={() => {
                onAcceptBid(item);
              }}
            >
              {!account?.genRoochAddress().toHexAddress()
                ? 'Please connect wallet'
                : new BigNumber(toCoinBalanceInfo.balance || 0).isLessThan(item.quantity)
                  ? 'Insufficient Balance'
                  : 'Accept Bid'}
            </Button>
          )}
        </Stack>
      </CardActions>
    </Card>
  );
}
