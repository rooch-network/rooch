import type {
  SelectChangeEvent} from '@mui/material';
import type { BalanceInfoView} from '@roochnetwork/rooch-sdk';

import { useMemo, useState } from 'react';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import {
  SessionKeyGuard,
  useCurrentAddress,
  useRoochClientQuery,
  useSignAndExecuteTransaction,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Stack,
  Dialog,
  Select,
  MenuItem,
  TextField,
  Typography,
  InputLabel,
  DialogTitle,
  FormControl,
  DialogContent,
  DialogActions,
} from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { isNumber } from 'src/utils/reg';
import { toDust, formatByIntl } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

export default function CreateLiquidityModal({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();
  const { mutateAsync, isPending } = useSignAndExecuteTransaction();

  const [x, setX] = useState<BalanceInfoView>();
  const [xValue, setXValue] = useState('');
  const [xAmount, setXAmount] = useState('');
  const [y, setY] = useState<BalanceInfoView>();
  const [yValue, setYValue] = useState('');
  const [yAmount, setYAmount] = useState('');

  const { data: balances } = useRoochClientQuery(
    'getBalances',
    {
      owner: currentAddress?.toStr() || '',
    },
    {
      refetchInterval: 5000,
    }
  );

  // map<coin_type, ...>
  const assetsMap = useMemo(() => {
    const assetsMap = new Map<string, BalanceInfoView>();
    balances?.data
      .filter((item) => !(item.symbol.startsWith('RDexLP') || item.symbol === 'BITXP'))
      .forEach((i) => {
        assetsMap.set(i.coin_type, {
          ...i,
        });
      });
    return assetsMap;
  }, [balances]);

  const handleCreateLiquidity = () => {
    if (!x || !y) {
      return;
    }

    const fixdXAmount = toDust(xAmount.replaceAll(',', ''), x!.decimals);
    const fixdYAmount = toDust(yAmount.replaceAll(',', ''), y!.decimals);
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::router::create_token_pair`,
      args: [
        Args.u64(fixdXAmount),
        Args.u64(fixdYAmount),
        Args.u64(fixdXAmount),
        Args.u64(fixdYAmount),
      ],
      typeArgs: [x.coin_type, y.coin_type],
    });
    mutateAsync({
      transaction: tx,
    })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          toast.success('create success');
        } else {
          console.log(result);
          toast.error('create failed');
        }
      })
      .catch((e: any) => {
        console.log(e);
        toast.error('create failed');
      })
      .finally(() => {
        onClose();
      });
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle sx={{ pb: 2 }}>Create Liqidity</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          height: '280px',
          overflow: 'unset',
        }}
      >
        <Stack direction="row" justifyContent="space-between" alignItems="center" mt={2}>
          <FormControl sx={{ minWidth: 120, mr: 1 }}>
            <InputLabel id="select-x">X</InputLabel>
            <Select
              labelId="select-x"
              id="select-x"
              key="select-x"
              value={xValue}
              label="X"
              onChange={(e: SelectChangeEvent) => {
                const {value} = e.target;
                setXValue(value);
                setX(assetsMap.get(value));
              }}
            >
              {assetsMap &&
                [...assetsMap.entries()].map(([key, pairs]) => (
                  <MenuItem key={key} id={key} value={`${key}`}>
                    <span>{pairs.symbol}</span>
                  </MenuItem>
                ))}
            </Select>
          </FormControl>
          <FormControl fullWidth>
            <TextField
              label="Initial liquidity"
              value={xAmount}
              inputMode="decimal"
              autoComplete="off"
              onChange={(e) => {
                const value = e.target.value.replaceAll(',', '');
                if (!isNumber(value)) {
                  return;
                }

                const nValue = Number(value) > x!.fixedBalance ? x!.fixedBalance : Number(value);
                setXAmount(formatByIntl(nValue));
              }}
            />
          </FormControl>
        </Stack>
        <Stack direction="row" justifyContent="flex-end" mt={1}>
          <Typography className="text-gray-600 !text-sm !font-semibold">
            {formatByIntl(x?.fixedBalance)}: Balance
          </Typography>
        </Stack>
        <Stack direction="row" justifyContent="space-between" alignItems="center" mt={4}>
          <FormControl sx={{ minWidth: 120, mr: 1 }}>
            <InputLabel id="select-y">X</InputLabel>
            <Select
              labelId="select-y"
              id="select-y"
              key="select-y"
              value={yValue}
              label="Y"
              onChange={(e: SelectChangeEvent) => {
                const {value} = e.target;
                setYValue(value);
                setY(assetsMap.get(value));
              }}
            >
              {assetsMap &&
                [...assetsMap.entries()]
                  .filter((item) => item[0] !== x?.coin_type)
                  .map(([key, pairs]) => (
                    <MenuItem key={key} id={key} value={`${key}`}>
                      <span>{pairs.symbol}</span>
                    </MenuItem>
                  ))}
            </Select>
          </FormControl>
          <FormControl fullWidth>
            <TextField
              label="Initial liquidity"
              value={yAmount}
              inputMode="decimal"
              autoComplete="off"
              onChange={(e) => {
                const value = e.target.value.replaceAll(',', '');
                if (!isNumber(value)) {
                  return;
                }

                const nValue = Number(value) > y!.fixedBalance ? y!.fixedBalance : Number(value);
                setYAmount(formatByIntl(nValue));
              }}
            />
          </FormControl>
        </Stack>
        <Stack direction="row" justifyContent="flex-end" mt={1}>
          <Typography className="text-gray-600 !text-sm !font-semibold">
            {formatByIntl(y?.fixedBalance)}: Balance
          </Typography>
        </Stack>
      </DialogContent>

      <DialogActions>
        <SessionKeyGuard onClick={handleCreateLiquidity}>
          <LoadingButton
            fullWidth
            loading={isPending}
            disabled={!x || !y || xAmount === '' || yAmount === ''}
            variant="contained"
          >
            Confirm
          </LoadingButton>
        </SessionKeyGuard>
      </DialogActions>
    </Dialog>
  );
}
