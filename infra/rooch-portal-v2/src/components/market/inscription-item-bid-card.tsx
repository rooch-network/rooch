import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import type { BidItem, MarketItem } from 'src/hooks/trade/use-market-data';

import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, UseSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { Card, Stack, Button, CardActions } from '@mui/material';

import { NETWORK, NETWORK_PACKAGE } from 'src/config/trade';

import { toast } from '../snackbar';
import InscriptionShopCard from './inscription-shop-card';

export type InscriptionItemCardProps = {
  item: MarketItem;
  tick: string;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  onAcceptBid: (item: BidItem) => void;
  onRefetchMarketData: () => Promise<void>;
};

export default function InscriptionItemBidCard({
  item,
  tick,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  onAcceptBid,
  onRefetchMarketData,
}: InscriptionItemCardProps) {
  const account = useCurrentAddress();
  const { mutate: signAndExecuteTransaction, isPending } = UseSignAndExecuteTransaction();

  const price = useMemo(
    () => new BigNumber(item.unit_price).times(item.quantity).toString(),
    [item.quantity, item.unit_price]
  );

  return (
    <Card
      key={item.order_id}
      sx={{
        '&:hover .add-cart-btn': {
          opacity: 1,
        },
        p: 1,
      }}
      onClick={() => {}}
    >
      <InscriptionShopCard
        fromCoinBalanceInfo={fromCoinBalanceInfo}
        toCoinBalanceInfo={toCoinBalanceInfo}
        objectId={item.order_id}
        tick={tick}
        isVerified
        amount={item.quantity}
        price={price}
        unitPrice={item.unit_price}
        acc="0"
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
          {account?.genRoochAddress().toStr() === item.owner ? (
            <LoadingButton
              loading={isPending}
              variant="outlined"
              color="warning"
              fullWidth
              size="small"
              onClick={() => {
                const tx = new Transaction();
                tx.callFunction({
                  target: `${NETWORK_PACKAGE[NETWORK].MARKET_PACKAGE_ID}::market::cancel_bid`,
                  args: [
                    Args.objectId(
                      NETWORK_PACKAGE[NETWORK].tickInfo[tick.toLowerCase()].MARKET_OBJECT_ID
                    ),
                    Args.objectId(item.order_id),
                  ],
                });
                signAndExecuteTransaction(
                  {
                    transaction: tx,
                  },
                  {
                    async onSuccess(data) {
                      // await refetchAddressOwnedInscription();
                      toast.success('Cancel Bid Success');
                      onRefetchMarketData();
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
              onClick={() => {
                onAcceptBid(item);
              }}
            >
              Accept Bid
            </Button>
          )}
        </Stack>
      </CardActions>
    </Card>
  );
}
