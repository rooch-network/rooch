import type { UserCoin } from 'src/components/swap/types';

import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { SessionKeyGuard, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Box,
  Card,
  Stack,
  Button,
  Dialog,
  CardHeader,
  DialogTitle,
  CardContent,
  DialogActions,
  DialogContent,
} from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { fNumber } from 'src/utils/format-number';
import { toDust, bigNumberToBigInt } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

export default function SwapConfirmModal({
  open,
  onClose,
  fromCoin,
  toCoin,
  slippage,
}: {
  open: boolean;
  onClose: () => void;
  fromCoin: UserCoin;
  toCoin: UserCoin;
  slippage: number;
}) {
  const dex = useNetworkVariable('dex');
  const { mutateAsync, isPending } = useSignAndExecuteTransaction();

  const handleSwap = () => {
    console.log(fromCoin, toCoin);
    if (!fromCoin || !toCoin) {
      return;
    }
    const tx = new Transaction();
    const fixedFromCoinAmount = toDust(fromCoin.amount.toString(), fromCoin.decimals);
    const fixedToCoinAmount = toDust(toCoin.amount.toString(), toCoin.decimals);

    const finalToCoinAmount =
      slippage !== 0
        ? bigNumberToBigInt(
            BigNumber(fixedToCoinAmount.toString()).minus(
              BigNumber(fixedToCoinAmount.toString()).multipliedBy(BigNumber(slippage))
            )
          )
        : fixedToCoinAmount;
    tx.callFunction({
      target: `${dex.address}::router::swap_with_exact_input`,
      args: [Args.u64(fixedFromCoinAmount), Args.u64(finalToCoinAmount)],
      typeArgs: [fromCoin.coin_type, toCoin.coin_type],
    });

    mutateAsync({ transaction: tx })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          toast.success('swap success');
        } else {
          toast.error('swap failed');
        }
      })
      .catch((e: any) => {
        console.log(e);
        toast.error('swap failed');
      })
      .finally(() => {
        onClose();
      });
  };

  return (
    <Dialog open={open}>
      <DialogTitle sx={{ pb: 2 }}>Swap Confirm</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          overflow: 'unset',
        }}
      >
        <Card>
          <CardHeader title="You balance change" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <Box sx={{ mt: 2 }}>
              <Stack direction="column" spacing="2">
                <Stack direction="row" alignItems="center" justifyContent="space-between">
                  <Box
                    component="img"
                    src={fromCoin.icon_url}
                    className="mr-1"
                    sx={{ width: 32, height: 32 }}
                  />
                  <Box className="text-gray-400 text-sm font-medium">
                    {' '}
                    - {fNumber(fromCoin.amount.toString(), { maximumFractionDigits: 16 })}
                    {` ${fromCoin.symbol} `}
                  </Box>
                </Stack>
                <Stack
                  direction="row"
                  alignItems="center"
                  justifyContent="space-between"
                  sx={{ mt: 2 }}
                >
                  <Box
                    component="img"
                    src={toCoin.icon_url}
                    className="mr-1"
                    sx={{ width: 32, height: 32 }}
                  />
                  <Box className="text-gray-400 text-sm font-medium">
                    {' '}
                    + {fNumber(toCoin.amount.toString(), { maximumFractionDigits: 16 })}
                    {` ${toCoin.symbol} `}
                  </Box>
                </Stack>
              </Stack>
              <Stack direction="row" alignItems="center" spacing={0.5} sx={{ mt: 2 }}>
                <Box className="text-gray-400 text-sm font-medium">Price :</Box>
                <Box>
                  1 {fromCoin.symbol} ≈{' '}
                  {fNumber(
                    new BigNumber(toCoin.amount.toString())
                      .div(fromCoin.amount.toString())
                      .toString(),
                    {
                      maximumFractionDigits: 8,
                    }
                  )}
                  {toCoin.symbol}
                </Box>
              </Stack>
              <Stack direction="row" alignItems="center" spacing={0.5} sx={{ mt: 2 }}>
                <Box className="text-gray-400 text-sm font-medium">Slippage :</Box>
                <Box>{slippage * 100}%</Box>
              </Stack>
            </Box>
          </CardContent>
        </Card>
      </DialogContent>

      <DialogActions>
        <Button
          fullWidth
          variant="outlined"
          color="inherit"
          onClick={() => {
            onClose();
          }}
        >
          Cancel
        </Button>

        <SessionKeyGuard onClick={handleSwap}>
          <LoadingButton fullWidth loading={isPending} variant="contained">
            Confirm
          </LoadingButton>
        </SessionKeyGuard>
      </DialogActions>
    </Dialog>
  );
}
