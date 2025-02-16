import { useMemo } from 'react';

import { Box, CircularProgress, Stack, Typography } from '@mui/material';

import { grey, secondary } from 'src/theme/core';
import swapDownIcon from '@/assets/swap/swap-down.svg';

import Label from './typography/label';
import { formatCoin } from '../../utils/number';

import type { UserCoin, InteractiveMode } from './types';

export interface SwapInfoCardProps {
  fromCoin: UserCoin;
  toCoin: UserCoin;
  interactiveMode: InteractiveMode;
  type?: 'propose' | 'pending' | 'history';
  loading?: boolean;
}

export default function SwapInfoCard({
  fromCoin,
  toCoin,
  interactiveMode,
  type = 'propose',
  loading,
}: SwapInfoCardProps) {
  return (
    <Stack alignItems="center" spacing={-1}>
      <SwapItem
        coin={fromCoin}
        interactiveMode={interactiveMode}
        type="from"
        variant={type}
        loading={loading}
      />
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: '32px',
          height: '32px',
          padding: '4px',
          borderRadius: '32px',
          border: '4px solid #FFF',
          background: secondary.light,
          zIndex: 1,
        }}
      >
        <Box component="img" src={swapDownIcon} width="100%" />
      </Box>
      <SwapItem
        coin={toCoin}
        interactiveMode={interactiveMode}
        type="to"
        variant={type}
        loading={loading}
      />
    </Stack>
  );
}

function SwapItem({
  coin,
  interactiveMode,
  type,
  variant,
  loading,
}: {
  coin: UserCoin;
  interactiveMode: InteractiveMode;
  type: 'from' | 'to';
  variant: 'propose' | 'pending' | 'history';
  loading?: boolean;
}) {
  const bgColor = useMemo(() => {
    if (variant === 'propose') {
      return interactiveMode === type ? '#FCFDFD' : '#F1F5F5';
    }
    return '#FFF';
  }, [interactiveMode, type, variant]);

  const imageSize = useMemo(() => (variant === 'propose' ? '32px' : '24px'), [variant]);

  const border = useMemo(
    () => (variant === 'propose' ? '1px solid #F1F5F5' : '1px solid #E2E4E9'),
    [variant]
  );

  const label = useMemo(() => {
    if (variant === 'propose' || variant === 'history') {
      return type.toUpperCase();
    }
    return interactiveMode === type ? type.toUpperCase() : `${type.toUpperCase()} (OBSERVED)`;
  }, [variant, interactiveMode, type]);

  return (
    <Stack
      spacing={variant === 'propose' ? 1 : '6px'}
      sx={{
        position: 'relative',
        padding: '8px 24px',
        borderRadius: '8px',
        border,
        background: bgColor,
        boxShadow: '0px 1px 2px 0px rgba(16, 24, 40, 0.05)',
        width: '100%',
      }}
    >
      <Label>{label}</Label>
      <Stack direction="row" alignItems="center">
        {loading ? (
          <CircularProgress size={variant === 'propose' ? '2.25rem' : '1.75rem'} />
        ) : (
          <Typography
            sx={{
              flexGrow: 1,
              fontSize: variant === 'propose' ? '2.25rem' : '1.75rem',
              fontWeight: 600,
              lineHeight: '100%',
              color: grey[900],
            }}
          >
            {formatCoin(coin)}
          </Typography>
        )}
        {!loading && (
          <Stack
            direction="row"
            alignItems="center"
            spacing={1}
            sx={{ '& img': { width: imageSize, height: imageSize } }}
          >
            <Box component="img" src={coin.icon} />
            <Typography
              sx={{
                fontSize: '1.125rem',
                fontWeight: 500,
                lineHeight: '24px',
                color: grey[900],
              }}
            >
              {coin.symbol}
            </Typography>
          </Stack>
        )}
      </Stack>
    </Stack>
  );
}
