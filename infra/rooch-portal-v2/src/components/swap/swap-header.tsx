import type { ReactNode } from 'react';

import { useMemo, useState } from 'react';

import { Box, Stack, Button, Popover, Tooltip, TextField, InputAdornment } from '@mui/material';

import { toBigNumber } from 'src/utils/number';

import { grey } from 'src/theme/core';

import Label from './typography/label';
import Header from './typography/header';
import { Iconify } from '../iconify/iconify';

export interface SwapWidgetHeaderProps {
  slippage: number;
  header?: ReactNode;
  fixedSwap?: boolean;
  onSlippageChange: (slippage: number) => void;
}

export default function SwapHeader({
  slippage,
  header = 'Purchase RGas',
  fixedSwap,
  onSlippageChange,
}: SwapWidgetHeaderProps) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const headerComponent = useMemo(() => {
    if (typeof header === 'string') {
      return (
        <Stack direction="row" alignItems="center" flexGrow={1}>
          <Stack direction="row" alignItems="center" spacing={1} sx={{ height: '44px' }}>
            <Header>{header}</Header>
          </Stack>
        </Stack>
      );
    }
    return header;
  }, [header]);

  const slippageHint = useMemo(() => {
    if (slippage < 0.005) {
      return 'Your transaction may fail';
    }
    if (slippage > 0.015) {
      return 'Your transaction may be frontrun';
    }
    return '';
  }, [slippage]);

  const displayValue = useMemo(() => toBigNumber(slippage).times(100).toFixed(3), [slippage]);

  return (
    <Stack
      direction="row"
      alignItems="center"
      sx={{
        borderRadius: '16px 16px 0 0',
        borderBottom: `1px solid ${grey[200]}`,
        background: grey[50],
        padding: '4px 30px',
      }}
    >
      {headerComponent}
      {!fixedSwap && (
        <>
          <Box
            component="img"
            src="assets/icons/swap/settings.svg"
            sx={{
              cursor: 'pointer',
              transition: 'all 0.3s ease',
              '&:hover': {
                transform: 'rotate(180deg)',
              },
            }}
            onClick={(e) => {
              setAnchorEl(e.currentTarget);
            }}
          />
          <Popover
            anchorEl={anchorEl}
            open={open}
            onClose={() => setAnchorEl(null)}
            anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
            transformOrigin={{ vertical: 'top', horizontal: 'right' }}
            sx={{
              '& .MuiPaper-root': {
                width: '300px',
                mt: '8px',
                borderRadius: '6px',
                border: '1px solid white',
                boxShadow: '0px 4px 8px 0px rgba(0, 0, 0, 0.15)',
              },
              '& .MuiList-root': {
                padding: 0,
              },
            }}
          >
            <Stack>
              <Stack
                direction="row"
                alignItems="center"
                sx={{
                  padding: '6px 12px',
                  borderBottom: '1px solid #EAECF0',
                }}
              >
                <Label sx={{ flexGrow: 1 }}>Settings</Label>
                <Box
                  component="img"
                  src="assets/icons/swap/chevron-down.svg"
                  sx={{ cursor: 'pointer' }}
                  onClick={() => setAnchorEl(null)}
                />
              </Stack>
              <Stack
                spacing={2}
                sx={{
                  padding: '16px 24px 24px 24px',
                }}
              >
                <Stack direction="row" spacing="10px" alignItems="center">
                  <Box component="img" src="assets/icons/swap/slippage-settings.svg" />
                  <Label sx={{ flexGrow: 1 }}>Max slippage</Label>
                  <Tooltip
                    title="Slippage refers to the difference between the expected price of a trade and the price at which the trade is executed. "
                    placement="right"
                  >
                    <Iconify icon="solar:question-circle-outline" width="16px" />
                  </Tooltip>
                </Stack>
                <Stack direction="row" spacing={1} justifyContent="space-between">
                  {[0.005, 0.01, 0.015].map((v) => (
                    <Button
                      key={v}
                      variant="outlined"
                      size="small"
                      onClick={() => onSlippageChange(v)}
                      sx={{ borderColor: slippage === v ? 'black' : 'gray', width: '100%' }}
                    >
                      {v * 100}%
                    </Button>
                  ))}
                </Stack>
                <TextField
                  value={displayValue}
                  type="number"
                  placeholder="0.5"
                  error={!!slippageHint}
                  helperText={slippageHint}
                  inputProps={{
                    min: 0,
                    max: 50,
                    step: 0.5,
                    style: {
                      textAlign: 'right',
                      width: '48%',
                    },
                  }}
                  InputProps={{
                    endAdornment: (
                      <InputAdornment
                        position="end"
                        sx={{
                          position: 'absolute',
                          left: '52%',
                        }}
                      >
                        %
                      </InputAdornment>
                    ),
                  }}
                  sx={{
                    'input::-webkit-outer-spin-button, input::-webkit-inner-spin-button': {
                      WebkitAppearance: 'none',
                      margin: 0,
                    },
                  }}
                  onChange={(e) => {
                    const result = Number(e.target.value);
                    if (result) {
                      onSlippageChange(toBigNumber(result).div(100).toNumber());
                    }
                  }}
                />
              </Stack>
            </Stack>
          </Popover>
        </>
      )}
    </Stack>
  );
}
