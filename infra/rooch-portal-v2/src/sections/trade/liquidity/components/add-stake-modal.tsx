import { useState } from 'react';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { SessionKeyGuard, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Stack,
  Button,
  Dialog,
  Typography,
  DialogTitle,
  DialogActions,
  DialogContent,
} from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { toDust, formatByIntl } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

import AmountInput from '../../components/amount-input';

import type { FarmRowItemType } from './farm-row-item';

export default function AddStakeModal({
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

  const handleStake = () => {
    const fixedAmount = toDust(amount.replaceAll(',', ''), row.liquidity!.decimals);
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::liquidity_incentive::stake`,
      args: [Args.u256(fixedAmount), Args.objectId(row.id)],
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
