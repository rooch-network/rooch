import DOMPurify from 'dompurify';
import BigNumber from 'bignumber.js';
import {
  Args,
  Transaction,
} from '@roochnetwork/rooch-sdk';
import {
  SessionKeyGuard,
  useSignAndExecuteTransaction,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { Box , Card ,
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

import { toDust, bigNumberToBigInt } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

import type { TradeCoinType } from '../components/types';

export default function SwapConfirmModal({
  open,
  onClose,
  x,
  y,
  slippage,
}: {
  open: boolean;
  onClose: () => void;
  x: TradeCoinType;
  y: TradeCoinType;
  slippage: number;
}) {
  const dex = useNetworkVariable('dex');
  const { mutateAsync, isPending } = useSignAndExecuteTransaction();

  const handleSwap = () => {
    if (!x || !y) {
      return;
    }
    const tx = new Transaction();
    const fixdX = toDust(x.amount, x.decimal);
    const fixdY = toDust(y.amount, y.decimal);

    const finalY =
      slippage !== 0
        ? bigNumberToBigInt(
            BigNumber(fixdY.toString()).minus(
              BigNumber(fixdY.toString()).multipliedBy(BigNumber(slippage))
            )
          )
        : fixdX;
    tx.callFunction({
      target: `${dex.address}::router::swap_with_exact_input`,
      args: [Args.u64(fixdX), Args.u64(finalY)],
      typeArgs: [x!.type, y!.type],
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
                    component="span"
                    className="mr-1"
                    sx={{ width: 32, height: 32 }}
                    dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(x.icon || '') }}
                  />
                  <Box className="text-gray-400 text-sm font-medium">
                    {' '}
                    - {x.amount}
                    {` ${x.symbol} `}
                  </Box>
                </Stack>
                <Stack
                  direction="row"
                  alignItems="center"
                  justifyContent="space-between"
                  sx={{ mt: 2 }}
                >
                  <Box
                    component="span"
                    className="mr-1"
                    sx={{ width: 32, height: 32 }}
                    dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(y.icon || '') }}
                  />
                  <Box className="text-gray-400 text-sm font-medium">
                    {' '}
                    + {y.amount}
                    {` ${y.symbol} `}
                  </Box>
                </Stack>
              </Stack>
              <Stack direction="row" alignItems="center" spacing={0.5} sx={{ mt: 2 }}>
                <Box className="text-gray-400 text-sm font-medium">Price :</Box>
                <Box>
                  1 {x.symbol} ≈ {`${BigNumber(y.amount).div(x.amount).toFixed(0)}`} {y.symbol}
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
