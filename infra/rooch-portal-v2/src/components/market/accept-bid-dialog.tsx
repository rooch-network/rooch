import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import type { BidItem } from 'src/hooks/trade/use-market-data';

import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { yellow } from '@mui/material/colors';
import {
  Card,
  Chip,
  Stack,
  Dialog,
  Button,
  Typography,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material';

import { fNumber } from 'src/utils/format-number';
import { formatUnitPrice } from 'src/utils/marketplace';
import { fromDust, formatNumber } from 'src/utils/number';

import { secondary } from 'src/theme/core';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import OrderShopCard from './order-shop-card';
import { useNetworkVariable } from '../../hooks/use-networks';

export type AcceptBidDialogProps = {
  open: boolean;
  acceptBidItem: BidItem;
  tokenBalance: string;
  tick: string;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  refreshBidList: () => Promise<void>;
  close: () => void;
};

export default function AcceptBidDialog({
  open,
  acceptBidItem,
  tick,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  tokenBalance,
  refreshBidList,
  close,
}: AcceptBidDialogProps) {
  const market = useNetworkVariable('market');
  const account = useCurrentAddress();
  const { mutate: signAndExecuteTransaction, isPending } = useSignAndExecuteTransaction();

  const price = useMemo(
    () =>
      new BigNumber(formatUnitPrice(acceptBidItem.unit_price, toCoinBalanceInfo.decimals))
        .times(fromDust(acceptBidItem.quantity, toCoinBalanceInfo.decimals))
        .toString(),
    [acceptBidItem.quantity, acceptBidItem.unit_price, toCoinBalanceInfo.decimals]
  );

  return (
    <Dialog
      open={open}
      onClose={close}
      sx={{
        '& .MuiDialog-paper': {
          minWidth: {
            xs: '360px',
            sm: '360px',
            md: '480px',
            lg: '480px',
          },
        },
      }}
    >
      <DialogTitle>Accept Bid</DialogTitle>

      <DialogContent>
        <Card
          variant="outlined"
          sx={{
            p: 2,
          }}
        >
          <OrderShopCard
            objectId={acceptBidItem.order_id}
            tick={tick}
            isVerified
            amount={acceptBidItem.quantity}
            price={price}
            unitPrice={formatUnitPrice(acceptBidItem.unit_price, toCoinBalanceInfo.decimals)}
            fromCoinBalanceInfo={fromCoinBalanceInfo}
            toCoinBalanceInfo={toCoinBalanceInfo}
            seller={acceptBidItem.owner}
            selectMode={false}
            type="list"
          />
        </Card>

        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
          sx={{ mt: 3, mb: 0.25 }}
        >
          <Chip
            label={
              <Stack direction="row" alignItems="center">
                <Iconify
                  icon="solar:wallet-bold"
                  color={yellow.A200}
                  width={18}
                  sx={{
                    mr: 0.5,
                  }}
                />
                <Typography
                  sx={{
                    fontWeight: 600,
                    fontSize: '0.875rem',
                  }}
                >
                  {tick.toUpperCase()} Balance:{' '}
                  {fNumber(fromDust(tokenBalance, toCoinBalanceInfo.decimals).toNumber())}
                </Typography>
              </Stack>
            }
            size="small"
            variant="filled"
            color="secondary"
          />
        </Stack>

        <Typography sx={{ mt: 3, mb: 0.5 }}>
          You will receive:{' '}
          <span
            style={{
              fontWeight: 600,
              fontSize: '1.25rem',
              color: secondary.light,
            }}
          >
            {new BigNumber(price).isNaN()
              ? '--'
              : formatNumber(fromDust(price, fromCoinBalanceInfo.decimals).toNumber())}
          </span>{' '}
          {fromCoinBalanceInfo.symbol}
        </Typography>
      </DialogContent>

      <DialogActions>
        <Button
          onClick={() => {
            close();
          }}
          variant="outlined"
          color="inherit"
        >
          Cancel
        </Button>
        <LoadingButton
          loading={isPending}
          disabled={!account || new BigNumber(acceptBidItem.quantity).isGreaterThan(tokenBalance)}
          onClick={() => {
            if (!account || new BigNumber(acceptBidItem.quantity).isGreaterThan(tokenBalance)) {
              return;
            }
            console.log('🚀 ~ file: accept-bid-dialog.tsx:237 ~ acceptBidItem:', acceptBidItem);

            const tx = new Transaction();
            tx.callFunction({
              target: `${market.orderBookAddress}::market_v2::accept_bid`,
              args: [
                Args.objectId(market.tickInfo[tick].obj),
                Args.u64(BigInt(acceptBidItem.order_id)),
                Args.address(acceptBidItem.owner),
                Args.bool(true),
                Args.address(account.genRoochAddress().toStr()),
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
                    toast.success('Accept bid success');
                    close();
                    refreshBidList();
                  } else {
                    toast.error('Accept bid Failed');
                  }
                },
                onError(error) {
                  toast.error(String(error));
                },
              }
            );
          }}
          variant="contained"
        >
          Submit
        </LoadingButton>
      </DialogActions>
    </Dialog>
  );
}
