import { useMemo } from 'react';

import { Typography } from '@mui/material';

import { formatCoin } from 'src/utils/number';

import type { UserCoin, InteractiveMode } from './types';

export interface SwapHintTextProps {
  fromCoin: UserCoin;
  toCoin: UserCoin;
  interactiveMode: InteractiveMode;
  amount: number;
}

export default function SwapHintText({
  fromCoin,
  toCoin,
  interactiveMode,
  amount,
}: SwapHintTextProps) {
  const hintText = useMemo(() => {
    if (!amount) {
      return null;
    }
    if (interactiveMode === 'from') {
      return (
        <Typography
          sx={{
            fontSize: '0.875rem',
            fontWeight: 400,
            lineHeight: '24px',
            color: '#667075',
          }}
        >
          Output is estimated. You will receive at least{' '}
          <Typography
            component="span"
            sx={{
              fontSize: '0.875rem',
              fontWeight: 400,
              lineHeight: '24px',
              color: '#101828',
            }}
          >
            {formatCoin(toCoin)} {toCoin.symbol}
          </Typography>{' '}
          or the transaction will revert.
        </Typography>
      );
    }
    return (
      <Typography
        sx={{
          fontSize: '0.875rem',
          fontWeight: 400,
          lineHeight: '24px',
          color: '#667075',
        }}
      >
        Input is estimated. You will send at least{' '}
        <Typography
          component="span"
          sx={{
            fontSize: '0.875rem',
            fontWeight: 400,
            lineHeight: '24px',
            color: '#101828',
          }}
        >
          {formatCoin(fromCoin)} {fromCoin.symbol}
        </Typography>{' '}
        or the transaction will revert.
      </Typography>
    );
  }, [interactiveMode, fromCoin, toCoin, amount]);

  return hintText;
}
