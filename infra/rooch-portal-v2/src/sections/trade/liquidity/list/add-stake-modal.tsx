import { useState } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { SessionKeyGuard, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
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

import { toast } from 'src/components/snackbar';

import type { FarmRowItemType } from './farm-row-item';
import AmountInput from '../../components/amount_input';

// TODO: 计算收入
export default function AddSrakeModal({
  open,
  onClose,
  row,
}: {
  open: boolean;
  onClose: () => void;
  row: FarmRowItemType;
}) {
  const dex = useNetworkVariable('dex');

  const { mutateAsync, isPending } = useSignAndExecuteTransaction();

  const [amount, setAmount] = useState('');
  const [slippage, setSlippage] = useState(0.005);
  const [customSlippage, setCustomSlippage] = useState('');

  const handleStake = () => {
    const fixdAmount = toDust(amount.replaceAll(',', ''), row.liquidity!.decimals);
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::liquidity_incentive::stake`,
      args: [Args.u256(fixdAmount), Args.objectId(row.id)],
      typeArgs: [row.x.type, row.y.type, row.reward],
    });
    mutateAsync({
      transaction: tx,
    })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          toast.success('stake success');
        } else {
          console.log(result);
          toast.error('stake failed');
        }
      })
      .catch((e: any) => {
        console.log(e);
        toast.error('stake failed');
      })
      .finally(() => {
        onClose();
      });
  };

  return (
    <Dialog open={open}>
      <DialogTitle sx={{ pb: 2 }}>Stake</DialogTitle>

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
            <Typography className="!font-semibold">
              {row.x.name}-{row.y.name}
            </Typography>
            <Typography className="text-gray-400 !text-xs">{row.liquidity?.symbol}</Typography>
          </Stack>
          <Stack>
            <Typography className="text-gray-600 !text-sm !font-semibold">
              Balance: {formatByIntl(row.liquidity?.fixedBalance)}
            </Typography>
          </Stack>
        </Stack>
        <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1 }}>
          <AmountInput
            max={row.liquidity?.fixedBalance || 0}
            amount={amount}
            onChange={(v) => setAmount(v)}
          />
        </Stack>
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

        <SessionKeyGuard onClick={handleStake}>
          <LoadingButton fullWidth disabled={amount === ''} loading={isPending} variant="contained">
            Confirm
          </LoadingButton>
        </SessionKeyGuard>
      </DialogActions>
    </Dialog>
  );
}
