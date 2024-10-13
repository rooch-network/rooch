import { useDebounce } from 'react-use';
import { useMemo, useState, useEffect } from 'react';

import { Stack, TextField } from '@mui/material';

import { toDust, fromDust, formatCoin, toBigNumber } from 'src/utils/number';

import { grey } from 'src/theme/core';

import Label from './typography/label';
import SwapCoinSelectButton from './swap-coin-select-button';

import type { UserCoin, InteractiveMode } from './types';

export interface SwapCoinInputProps {
  coins: UserCoin[];
  coin?: UserCoin;
  type: InteractiveMode;
  interactiveMode: InteractiveMode;
  disabledCoins: string[];
  fixedSwap?: boolean;
  hiddenValue?: boolean;
  onChange: (coin: UserCoin, source: 'amount' | 'coin') => void;
}

export default function SwapCoinInput({
  coins,
  coin,
  type,
  interactiveMode,
  disabledCoins,
  fixedSwap,
  hiddenValue,
  onChange,
}: SwapCoinInputProps) {
  const [value, setValue] = useState('');
  const [debouncedValue, setDebouncedValue] = useState(coin?.amount || 0n);
  const [shouldUpdate, setShouldUpdate] = useState(false);

  useDebounce(
    () => {
      if (coin) {
        try {
          let temp = value;
          const decimalIndex = value.indexOf('.');
          if (decimalIndex !== -1 && value.length - decimalIndex - 1 > coin.decimals) {
            temp = value.substring(0, decimalIndex + coin.decimals + 1);
          }
          const amount = toDust(temp, coin.decimals);
          setDebouncedValue(amount);
        } catch (e) {
//          toast.error(String(e));
            console.log(e)
        }
      }
    },
    300,
    [value, coin]
  );

  useEffect(() => {
    if (coin) {
      setShouldUpdate(false);
      setValue(fromDust(coin.amount, coin.decimals).decimalPlaces(coin.decimals).toString());
    }
  }, [coin, coin?.amount]);

  useEffect(() => {
    if (coin && shouldUpdate && toBigNumber(debouncedValue).gt(0)) {
      onChange(
        {
          ...coin,
          amount: debouncedValue,
        },
        'amount'
      );
    }
  }, [coin, debouncedValue, onChange, shouldUpdate]);

  const active = useMemo(() => type === interactiveMode, [type, interactiveMode]);
  const coinUsd = useMemo(() => {
    if (!coin) {
      return 0;
    }
    return fromDust(coin.amount, coin.decimals).times(coin.price).decimalPlaces(2).toNumber();
  }, [coin]);

  return (
    <Stack
      spacing={1}
      sx={{
        padding: '8px 24px',
        borderRadius: '8px',
        border: active ? '1px solid #F1F5F5' : `1px solid ${grey[200]}`,
        background: active ? '#FCFDFD' : '#F1F5F5',
        boxShadow: '0px 1px 2px 0px rgba(16, 24, 40, 0.05)',
        width: '100%',
      }}
    >
      <Stack direction="row">
        <Label sx={{ flexGrow: 1 }}>{type.toUpperCase()}</Label>
        {coin && (
          <Label
            onClick={() => {
              setShouldUpdate(true);
              setDebouncedValue(coin.balance);
            }}
            sx={{ cursor: 'pointer' }}
          >
            Balance: {formatCoin(coin, true)}
          </Label>
        )}
      </Stack>

      <Stack direction="row" alignItems="center" spacing={1}>
        <Stack spacing={0.25} sx={{ flexGrow: 1 }}>
          <TextField
            value={value}
            disabled={type === 'to' && fixedSwap}
            onChange={(e) => {
              setValue(e.target.value);
              setShouldUpdate(true);
            }}
            placeholder="0.0"
            type="text"
            autoComplete="off"
            inputProps={{
              inputMode: 'decimal',
              autoCorrect: 'off',
              pattern: '^[0-9]*[.,]?[0-9]*$',
              spellCheck: 'false',
            }}
            sx={{
              flexGrow: 1,
              '& .MuiInputBase-input': {
                padding: '0',
                fontSize: '1.25rem',
                fontWeight: 600,
                lineHeight: '24px',
                color: grey[900],
              },
              '& .MuiInputBase-input.Mui-disabled': {
                WebkitTextFillColor: grey[900],
              },
              '& fieldset': {
                border: 'none',
              },
            }}
          />
          {!hiddenValue && (
            <Stack direction="row" spacing={1} alignItems="center">
              <Label>$ {coinUsd}</Label>
            </Stack>
          )}
        </Stack>
        <SwapCoinSelectButton
          coins={coins}
          coin={coin}
          disabledCoins={disabledCoins}
          fixedSwap={fixedSwap}
          onSelect={(coin) => onChange(coin, 'coin')}
        />
      </Stack>
    </Stack>
  );
}
