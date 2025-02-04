import type { SelectChangeEvent } from '@mui/material';
import type { BalanceInfoView, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

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

export default function CreateTokenPair() {
  const client = useRoochClient();
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();

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
            onChange={(e: SelectChangeEvent) => {}}
          >
            {assetsMap &&
              [...assetsMap.entries()].map(([key, pairs]) => (
                <MenuItem key={key} id={key} value={`${key}`}>
                  <span>{key} :</span>
                  <span>{formatByIntl(pairs.fixedBalance || 0)}</span>
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
              // const xBalance = assetsMap?.get(x!.type)!.fixedBalance || 0;
              // if (xRatio !== 0) {
              //   if (value !== (xBalance * xRatio).toString()) {
              //     setXRation(0);
              //   }
              // }

              // if (Number(value) > xBalance) {
              //   setXCount(xBalance.toString());
              // } else {
              //   setXCount(value);
              // }
            }}
          />
        </FormControl>
      </Box>
      <Box display="flex" alignItems="center" justifyContent="space-between" sx={{ mt: 2 }}>
        <Box display="flex" alignItems="center">
          {[0.25, 0.5, 0.75, x?.name === 'RGas' ? 0.99 : 1].map((item, index) => (
            <Button
              key={item.toString()}
              variant={xRatio === item ? 'contained' : 'outlined'}
              size="small"
              sx={{ mx: 0.5 }}
              disabled={!x}
              onClick={() => {
                setXRation(item);
                const ration = item === 1 ? 0.99 : item; // TODO: Calculating gas
                setXCount(formatByIntl((assetsMap?.get(x!.type)!.fixedBalance || 0) * ration));
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
            onChange={(e: SelectChangeEvent) => {}}
          >
            {/* {assetsMap?.get(x?.name || '')?.map((item) => (
              <MenuItem key={item.y.name} id={item.y.name} value={`${item.y.name}`}>
                <span>{item.y.name} :</span>
                <span>{formatByIntl(assetsMap?.get(item.y.type)?.fixedBalance || 0)}</span>
              </MenuItem>
            ))} */}
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
