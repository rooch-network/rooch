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

import { toDust, fromDust } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

import type { TradeCoinType } from './types';

type TokenType = {
  id: string;
  type: string;
  name: string;
};

type TokenPairType = {
  x2y: boolean;
  x: TokenType;
  y: TokenType;
};

interface SelectTokenPairProps {
  onLoading: (status: boolean) => void;
  onCallback: (x?: TradeCoinType, y?: TradeCoinType) => void;
}

export default function SelectTokenPair({ onLoading, onCallback }: SelectTokenPairProps) {
  const client = useRoochClient();
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();

  const [loading, setLoading] = useState(false);
  const [x, setX] = useState<TokenType>();
  const [xValue, setXValue] = useState('');
  const [xCount, setXCount] = useState('');
  const [xRatio, setXRation] = useState(0);
  const [y, setY] = useState<TokenType>();
  const [x2y, setX2y] = useState(true);
  const [yValue, setYValue] = useState('');
  const [yCount, setYCount] = useState('');
  // map<x_coin_id, ...>
  const [tokenPair, setTokenPair] = useState<Map<string, TokenPairType[]>>();

  const { data } = useRoochClientQuery(
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
    data?.data.forEach((i) => {
      assetsMap.set(i.coin_type, {
        ...i,
      });
    });
    return assetsMap;
  }, [data]);

  useEffect(() => {
    client
      .queryObjectStates({
        filter: {
          object_type: `${dex.address}::swap::TokenPair`,
        },
      })
      .then((result) => {
        const pair: TokenPairType[] = result.data.map((item) => {
          const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
          let xType = xView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
          xType = xType.replace('>>', '');
          const xName = xType.split('::');
          const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
          let yType = yView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
          yType = yType.replace('>>', '');
          const yName = yType.split('::');
          return {
            x2y: true,
            x: {
              id: xView.value.id as string,
              type: xType,
              name: xName[xName.length - 1].replace('>>', ''),
            },
            y: {
              id: yView.value.id as string,
              type: yType,
              name: yName[yName.length - 1].replace('>>', ''),
            },
          };
        });

        const pairMap = new Map<string, TokenPairType[]>();
        pair.forEach((p) => {
          const key = p.x.name;
          if (!pairMap.has(key)) {
            pairMap.set(key, []);
          }
          pairMap.get(key)!.push(p);

          const key1 = p.y.name;
          if (!pairMap.has(key1)) {
            pairMap.set(key1, []);
          }
          pairMap.get(key1)!.push({
            x2y: false,
            x: p.y,
            y: p.x,
          });
        });

        // Update the state
        setTokenPair(pairMap);
      });
  }, [client, dex]);

  const fetchY = useCallback(async () => {
    if (xCount === '' || xCount === '0' || !x || !y) {
      return;
    }

    try {
      setLoading(true);
      const fixdXCount = toDust(xCount, assetsMap?.get(x.type)?.decimals || 0);
      const result = await client.executeViewFunction({
        target: `${dex.address}::router::get_amount_out`,
        args: [Args.u64(fixdXCount)],
        typeArgs: [x.type, y.type],
      });

      if (result.vm_status !== 'Executed') {
        toast.error('unknow error');
      }

      const yCount = result.return_values![0].decoded_value as string;
      const fixdYCount = fromDust(yCount, assetsMap?.get(y.type)?.decimals || 0);
      setYCount(fixdYCount.toString());

      const xCoin = assetsMap?.get(x.type)!;
      const yCoin = assetsMap?.get(y.type)!;
      onCallback(
        {
          balance: xCoin.fixedBalance,
          type: xCoin.coin_type,
          icon: xCoin.icon_url || undefined,
          symbol: xCoin.symbol,
          amount: xCount,
          decimal: xCoin.decimals,
        },
        {
          balance: yCoin.fixedBalance,
          type: yCoin.coin_type,
          icon: yCoin.icon_url || undefined,
          symbol: yCoin.symbol,
          amount: fixdYCount.toString(),
          decimal: yCoin.decimals,
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

    const xbalance = assetsMap?.get(oldY.type)!.fixedBalance || 0;
    if (xRatio !== 0) {
      setXCount((xbalance * xRatio).toString());
    } else if (Number(xCount) > xbalance) {
      setXCount(xbalance.toString());
      setXRation(0);
    }
  };

  useDebounce(fetchY, 500, [xCount, y]);

  return (
    <>
      <Box sx={{ display: 'flex', alignItems: 'center', mt: 4 }}>
        <FormControl sx={{ minWidth: 220 }}>
          <InputLabel id="select-x">X</InputLabel>
          <Select
            labelId="select-x"
            id="select-x"
            key="select-x"
            value={xValue}
            label="X"
            onChange={(e: SelectChangeEvent) => {
              const s = e.target.value;
              setX(tokenPair!.get(s)![0].x);
              setXValue(e.target.value);
              setY(undefined);
              setYValue('');
              setYCount('');
              if (xRatio !== 0) {
                setXCount(((assetsMap?.get(x!.type)!.fixedBalance || 0) * xRatio).toString());
              }
            }}
          >
            {tokenPair &&
              [...tokenPair.entries()].map(([key, pairs]) => (
                <MenuItem key={key} id={key} value={`${key}`}>
                  <span>{key} :</span>
                  <span>{assetsMap?.get(pairs[0].x.type)?.fixedBalance || 0}</span>
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
              const xBalance = assetsMap?.get(x!.type)!.fixedBalance || 0;
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
        <Box sx={{ minWidth: 220, display: 'flex', justifyContent: 'center' }}>
          <Button
            startIcon={<ArrowDownwardIcon />}
            variant="text"
            disabled={!x || !y}
            onClick={exchange}
            sx={{ display: { xs: 'none', sm: 'flex', justifyContent: 'center' } }}
          >
            Exchange
          </Button>
        </Box>
        <Box display="flex" alignItems="center">
          {[0.25, 0.5, 0.75, 1].map((item, index) => (
            <Button
              key={item.toString()}
              variant={xRatio === item ? 'contained' : 'outlined'}
              size="small"
              sx={{ mx: 0.5 }}
              disabled={!x}
              // loading={loading && xRatio === item}
              onClick={() => {
                setXRation(item);
                setXCount(((assetsMap?.get(x!.type)!.fixedBalance || 0) * item).toString());
              }}
            >
              {item * 100}%
            </Button>
          ))}
        </Box>
      </Box>
      <Box sx={{ display: 'flex', alignItems: 'center', mt: 2 }}>
        <FormControl sx={{ minWidth: 220 }}>
          <InputLabel id="select-y">Y</InputLabel>
          <Select
            labelId="select-y"
            id="select-y"
            key="select-y"
            value={yValue}
            label="Y"
            onChange={(e: SelectChangeEvent) => {
              const s = e.target.value;
              const pair = tokenPair!.get(x!.name)!.find((item) => item.y.name === s)!;
              setY(pair.y);
              setYValue(e.target.value);
            }}
          >
            {tokenPair?.get(x?.name || '')?.map((item) => (
              <MenuItem key={item.y.name} id={item.y.name} value={`${item.y.name}`}>
                <span>{item.y.name} :</span>
                <span>{assetsMap?.get(item.y.type)?.fixedBalance || 0}</span>
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
