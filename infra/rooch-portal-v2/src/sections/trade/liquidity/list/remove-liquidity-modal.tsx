import { useState } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { SessionKeyGuard, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Box,
  Stack,
  Button,
  Dialog,
  TextField,
  Typography,
  DialogTitle,
  FormControl,
  DialogActions,
  DialogContent,
  InputAdornment,
} from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatByIntl, toDust } from 'src/utils/number';
import { formatCoin } from 'src/utils/format-number';

import { toast } from 'src/components/snackbar';

import type { OwnerLiquidityItemType } from '../../hooks/use-owner-liquidity';
import AmountInput from '../../components/amount_input';

// TODO: 计算收入
export default function RemoveLiquidityModal({
  open,
  onClose,
  row,
}: {
  open: boolean;
  onClose: () => void;
  row: OwnerLiquidityItemType;
}) {
  const dex = useNetworkVariable('dex');

  const { mutateAsync, isPending } = useSignAndExecuteTransaction();

  const [liquidity, setLiquidity] = useState('');
  const [slippage, setSlippage] = useState(0.005);
  const [customSlippage, setCustomSlippage] = useState('');

  const handleRemoveLiquidity = () => {
    const fixdLiquidity = toDust(liquidity.replaceAll(',', ''), row.decimals);
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::router::remove_liquidity`,
      args: [Args.u64(fixdLiquidity), Args.u64(BigInt(0)), Args.u64(BigInt(0))],
      typeArgs: [row.x.type, row.y.type],
    });
    mutateAsync({
      transaction: tx,
    })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          toast.success('remove success');
        } else {
          console.log(result);
          toast.error('remove failed');
        }
      })
      .catch((e: any) => {
        console.log(e);
        toast.error('remove failed');
      })
      .finally(() => {
        onClose();
      });
  };

  return (
    <Dialog open={open}>
      <DialogTitle sx={{ pb: 2 }}>Remove Liquidity</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          overflow: 'unset',
        }}
      >
        <Stack
          direction="row"
          className="mb-2 w-full"
          justifyContent="space-between"
          alignItems="flex-end"
        >
          <Stack>
            <Typography className="!font-semibold">{row.symbol}</Typography>
            <Typography className="text-gray-400 !text-xs">{row.name}</Typography>
          </Stack>
          <Stack>
            <Typography className="text-gray-600 !text-sm !font-semibold">
              Balance: {formatByIntl(row.fixedBalance)}
            </Typography>
          </Stack>
        </Stack>
        <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1 }}>
          <AmountInput
            max={row.fixedBalance}
            amount={liquidity}
            onChange={(v) => setLiquidity(v)}
          />
        </Stack>
        <Box sx={{ pt: 2, mt: 2 }}>
          <span className="text-gray-400 text-sm mt-4 mr-2">Slippage</span>
          {[0.005, 0.01, 0.03].map((item, index) => (
            <Button
              key={item.toString()}
              variant={slippage === item ? 'contained' : 'outlined'}
              size="small"
              sx={{ mr: 1 }}
              onClick={() => {
                if (slippage === item) {
                  setSlippage(0);
                } else {
                  setSlippage(item);
                  setCustomSlippage('');
                }
              }}
            >
              {item * 100}%
            </Button>
          ))}
        </Box>
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

        <SessionKeyGuard onClick={handleRemoveLiquidity}>
          <LoadingButton
            fullWidth
            disabled={liquidity === ''}
            loading={isPending}
            variant="contained"
          >
            Confirm
          </LoadingButton>
        </SessionKeyGuard>
      </DialogActions>
    </Dialog>
  );
}
