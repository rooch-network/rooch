import type { SelectChangeEvent } from '@mui/material';
import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { useDebounce } from 'react-use';
import { Args } from '@roochnetwork/rooch-sdk';
import { useMemo, useState, useEffect, useCallback } from 'react';
import {
  useRoochClient,
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import ArrowDownwardIcon from '@mui/icons-material/ArrowDownward';
import { Box, Button, Select, MenuItem, TextField, InputLabel, FormControl } from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { toDust, fromDust, formatByIntl } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

import { useTokenPair } from '../hooks/use-token-pair';

interface SelectTokenPairProps {
  onLoading: (status: boolean) => void;
  onCallback: (
    x?: { amount: string } & BalanceInfoView,
    y?: { amount: string } & BalanceInfoView
  ) => void;
}

export default function SelectTokenPair({ onLoading, onCallback }: SelectTokenPairProps) {
  const client = useRoochClient();
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();
  const { tokenPairs } = useTokenPair();

  const [loading, setLoading] = useState(false);
  const [x, setX] = useState<BalanceInfoView>();
  const [xValue, setXValue] = useState('');
  const [xCount, setXCount] = useState('');
  const [xRatio, setXRation] = useState(0);
  const [y, setY] = useState<BalanceInfoView>();
  const [x2y, setX2y] = useState(true);
  const [yValue, setYValue] = useState('');
  const [yCount, setYCount] = useState('');

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
    balances?.data.forEach((i) => {
      assetsMap.set(i.coin_type, {
        ...i,
      });
    });
    return assetsMap;
  }, [balances]);

  const fetchY = useCallback(async () => {
    if (xCount === '' || xCount === '0' || !x || !y) {
      return;
    }

    try {
      setLoading(true);
      const fixedXCount = toDust(
        xCount.replaceAll(',', ''),
        assetsMap?.get(x.coin_type)?.decimals || 0
      );
      const result = await client.executeViewFunction({
        target: `${dex.address}::router::get_amount_out`,
        args: [Args.u64(fixedXCount)],
        typeArgs: [x.coin_type, y.coin_type],
      });

      if (result.vm_status !== 'Executed') {
        toast.error('unknown error');
      }

      const yCount = result.return_values![0].decoded_value as string;
      const fixedYCount = fromDust(yCount, assetsMap?.get(y.coin_type)?.decimals || 0);
      setYCount(formatByIntl(fixedYCount.toString()));
      onCallback(
        {
          ...x,
          amount: xCount.replaceAll(',', ''),
        },
        {
          ...y,
          amount: fixedYCount.toString(),
        }
      );
    } catch (e) {
      console.log(e);
    } finally {
      setLoading(false);
    }
  }, [x, y, xCount, client, assetsMap, dex.address, onCallback]);

  useEffect(() => {
    const interval = setInterval(() => {
      fetchY();
    }, 2000);
    return () => clearInterval(interval);
  }, [fetchY]);

  const exchange = () => {
    const oldX = x;
    const oldXValue = xValue;
    const oldY = y;
    const oldYValue = yValue;

    if (!oldX || !oldY) {
      return;
    }
    setX2y(!x2y);
    setX(oldY);
    setXValue(oldYValue);
    setY(oldX);
    setYValue(oldXValue);

    const xBalance = assetsMap?.get(oldY.coin_type)!.fixedBalance || 0;
    if (xRatio !== 0) {
      setXCount(formatByIntl(xBalance * xRatio));
    } else if (Number(xCount) > xBalance) {
      setXCount(formatByIntl(xBalance));
      setXRation(0);
    }
  };

  useDebounce(fetchY, 500, [xCount, y]);

  return (
    <>
      <Box sx={{ display: 'flex', alignItems: 'center', mt: 4 }}>
        <FormControl sx={{ minWidth: 300 }}>
          <InputLabel id="select-x">X</InputLabel>
          <Select
            labelId="select-x"
            id="select-x"
            key="select-x"
            value={xValue}
            label="X"
            onChange={(e: SelectChangeEvent) => {
              const s = e.target.value;
              const x = tokenPairs!.get(s)?.x;
              setX(x);
              setXValue(e.target.value);
              setY(undefined);
              setYValue('');
              setYCount('');
              if (xRatio !== 0) {
                setXCount(formatByIntl((assetsMap?.get(x!.coin_type)?.fixedBalance || 0) * xRatio));
              }
            }}
          >
            {tokenPairs &&
              [...tokenPairs.entries()].map(([key, pairs]) => (
                <MenuItem key={key} id={key} value={`${key}`}>
                  <span>{key} :</span>
                  <span>
                    {formatByIntl(assetsMap?.get(pairs.x.coin_type)?.fixedBalance || 0, '0')}
                  </span>
                </MenuItem>
              ))}
          </Select>
        </FormControl>

        <FormControl fullWidth sx={{ ml: 2 }}>
          <TextField
            id="outlined-basic"
            label="Amount"
            value={xCount}
            variant="outlined"
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
              const { value } = e.target;
              // Use a regular expression to allow only whole numbers
              if (/^\d*\.?\d*$/.test(value) === false) {
                return;
              }
              const xBalance = assetsMap?.get(x!.coin_type)!.fixedBalance || 0;
              if (xRatio !== 0) {
                if (value !== (xBalance * xRatio).toString()) {
                  setXRation(0);
                }
              }

              if (Number(value) > xBalance) {
                setXCount(xBalance.toString());
              } else {
                setXCount(value);
              }
            }}
          />
        </FormControl>
      </Box>
      <Box display="flex" alignItems="center" justifyContent="space-between" sx={{ mt: 2 }}>
        <Box sx={{ minWidth: 300, display: 'flex', justifyContent: 'center' }}>
          <Button
            startIcon={<ArrowDownwardIcon />}
            variant="text"
            disabled={!x || !y || assetsMap.get(y.coin_type)?.fixedBalance === 0}
            onClick={exchange}
            sx={{ display: { xs: 'none', sm: 'flex', justifyContent: 'center' } }}
          >
            Exchange
          </Button>
        </Box>
        <Box display="flex" alignItems="center">
          {[0.25, 0.5, 0.75, x?.symbol === 'RGAS' ? 0.99 : 1].map((item, index) => (
            <Button
              key={item.toString()}
              variant={
                xRatio === item || (xRatio === 0.99 && item === 1) ? 'contained' : 'outlined'
              }
              size="small"
              sx={{ mx: 0.5 }}
              disabled={!x}
              onClick={() => {
                setXRation(item);
                const ration = item === 1 ? 0.99 : item; // TODO: Calculating gas
                setXCount(
                  formatByIntl((assetsMap?.get(x!.coin_type)?.fixedBalance || 0) * ration, '0')
                );
              }}
            >
              {item * 100}%
            </Button>
          ))}
        </Box>
      </Box>
      <Box sx={{ display: 'flex', alignItems: 'center', mt: 2 }}>
        <FormControl sx={{ minWidth: 300 }}>
          <InputLabel id="select-y">Y</InputLabel>
          <Select
            labelId="select-y"
            id="select-y"
            key="select-y"
            value={yValue}
            label="Y"
            onChange={(e: SelectChangeEvent) => {
              const s = e.target.value;
              const pair = tokenPairs!.get(x!.symbol)!.y.find((item) => item.symbol === s)!;
              setY(pair);
              setYValue(e.target.value);
            }}
          >
            {tokenPairs?.get(x?.symbol || '')?.y?.map((item) => (
              <MenuItem key={item.symbol} id={item.symbol} value={`${item.symbol}`}>
                <span>{item.symbol} :</span>
                <span>{formatByIntl(assetsMap?.get(item.coin_type)?.fixedBalance || 0, '0')}</span>
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl fullWidth sx={{ ml: 2 }}>
          <TextField
            id="outlined-basic"
            label={`${loading ? 'Refresh' : 'Automatic calculation'}`}
            disabled
            value={yCount}
            variant="outlined"
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
              setYCount(e.target.value);
            }}
          />
        </FormControl>
      </Box>
    </>
  );
}
