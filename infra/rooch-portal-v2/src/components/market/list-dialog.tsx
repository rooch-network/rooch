import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { useState } from 'react';
import BigNumber from 'bignumber.js';
import PuffLoader from 'react-spinners/PuffLoader';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { SessionKeyGuard, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { grey } from '@mui/material/colors';
import {
  Card,
  Stack,
  Dialog,
  Button,
  TextField,
  Typography,
  DialogTitle,
  DialogContent,
  DialogActions,
  InputAdornment,
} from '@mui/material';

import { toDust, fromDust, formatNumber } from 'src/utils/number';

import { warning, secondary } from 'src/theme/core';

import { toast } from 'src/components/snackbar';

import OrderCard from './order-card';
import { useNetworkVariable } from '../../hooks/use-networks';

export default function ListDialog({
  listDialogOpen,
  floorPrice,
  tick,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  refreshList,
  close,
}: {
  listDialogOpen: boolean;
  floorPrice?: string;
  tick: string;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  refreshList: () => Promise<void>;
  close: () => void;
}) {
  const market = useNetworkVariable('market');
  const { mutate: signAndExecuteTransaction, isPending } = useSignAndExecuteTransaction();

  const [listPrice, setListPrice] = useState('');
  const [listAmount, setListAmount] = useState('');

  const handleList = () => {
    if (!fromCoinBalanceInfo || !toCoinBalanceInfo) {
      return;
    }
    const tx = new Transaction();

    tx.callFunction({
      target: `${market.orderBookAddress}::market_v2::list`,
      args: [
        Args.objectId(market.tickInfo[tick].obj),
        Args.u256(
          BigInt(
            new BigNumber(toDust(listAmount, toCoinBalanceInfo.decimals).toString()).toNumber()
          )
        ),
        Args.u64(
          BigInt(
            new BigNumber(toDust(listPrice, fromCoinBalanceInfo.decimals).toString())
              .times(new BigNumber(10).pow(5))
              .div(new BigNumber(10).pow(toCoinBalanceInfo.decimals).toNumber())
              .toFixed()
          )
        ),
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
            toast.success('List success');
            close();
            refreshList();
          } else {
            toast.error('List Failed');
          }
        },
        onError(error) {
          toast.error(String(error));
        },
      }
    );
  };

  return (
    <Dialog
      open={listDialogOpen}
      onClose={close}
      sx={{
        '& .MuiDialog-paper': {
          minWidth: {
            xs: '360px',
            sm: '360px',
            md: '600px',
            lg: '600px',
          },
        },
      }}
    >
      <DialogTitle>List Token</DialogTitle>

      <DialogContent>
        {fromCoinBalanceInfo && toCoinBalanceInfo && (
          <Card
            variant="outlined"
            sx={{
              p: 2,
            }}
          >
            <OrderCard
              isVerified
              tick={toCoinBalanceInfo.symbol.toUpperCase()}
              tokenBalance={formatNumber(
                fromDust(toCoinBalanceInfo.balance, toCoinBalanceInfo.decimals).toNumber()
              )}
            />
          </Card>
        )}

        <Typography sx={{ mt: 3, mb: 0.5 }}>Price</Typography>

        <TextField
          autoFocus
          fullWidth
          type="number"
          autoComplete="off"
          InputProps={{
            endAdornment: (
              <InputAdornment position="end">
                {fromCoinBalanceInfo.symbol} / {toCoinBalanceInfo.symbol}
              </InputAdornment>
            ),
          }}
          margin="dense"
          value={listPrice}
          onChange={(e) => {
            setListPrice(e.target.value);
          }}
        />
        <Typography sx={{ mt: 3, mb: 0.5 }}>Amount</Typography>

        <TextField
          autoFocus
          fullWidth
          type="number"
          autoComplete="off"
          margin="dense"
          value={listAmount}
          onChange={(e) => {
            setListAmount(e.target.value);
          }}
        />
        {floorPrice !== undefined && (
          <Stack
            sx={{
              mt: 0.5,
              cursor: 'pointer',
            }}
            direction="row"
            alignItems="center"
            onClick={() => {
              setListPrice(floorPrice.toString());
            }}
            spacing={0.5}
          >
            <PuffLoader speedMultiplier={0.875} color={warning.light} loading size={16} />
            <Typography
              sx={{
                color: grey[500],
                fontSize: '0.875rem',
              }}
            >
              Latest floor price:{' '}
              <span
                style={{
                  color: secondary.light,
                }}
              >
                {new BigNumber(floorPrice).isNaN()
                  ? '--'
                  : formatNumber(fromDust(floorPrice, fromCoinBalanceInfo.decimals).toNumber())}
              </span>{' '}
              {fromCoinBalanceInfo.symbol}/{toCoinBalanceInfo.symbol}
            </Typography>
          </Stack>
        )}
        {fromCoinBalanceInfo && toCoinBalanceInfo && (
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Typography
              sx={{
                mt: 1,
              }}
            >
              Total Price:{' '}
              <span
                style={{
                  fontWeight: 600,
                  fontSize: '1.25rem',
                  color: secondary.light,
                }}
              >
                {new BigNumber(listPrice).times(listAmount).isNaN()
                  ? '-'
                  : new BigNumber(listPrice).times(listAmount).toFixed(4)}
              </span>{' '}
              {fromCoinBalanceInfo.symbol}
            </Typography>
          </Stack>
        )}
      </DialogContent>

      <DialogActions>
        <Button onClick={close} variant="outlined" color="inherit">
          Cancel
        </Button>
        <SessionKeyGuard onClick={handleList}>
          <LoadingButton
            loading={isPending}
            disabled={
              new BigNumber(listPrice).times(listAmount || 0).isNaN() ||
              new BigNumber(listPrice).isNaN() ||
              new BigNumber(listPrice).isZero()
            }
            variant="contained"
          >
            Submit
          </LoadingButton>
        </SessionKeyGuard>
      </DialogActions>
    </Dialog>
  );
}
