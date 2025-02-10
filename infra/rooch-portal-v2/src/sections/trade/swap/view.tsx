'use client';

import { useState } from 'react';
import BigNumber from 'bignumber.js';

import { Box, Stack, Button, TextField, Typography, FormControl } from '@mui/material';

import { isNumber } from 'src/utils/reg';
import { formatByIntl } from 'src/utils/number';

import { DashboardContent } from 'src/layouts/dashboard';

import SwapConfirmModal from './confirm-modal';
import SelectTokenPair from '../components/select-token-pair';

import type { TradeCoinType } from '../components/types';

export default function SwapView() {
  const [x, setX] = useState<TradeCoinType>();
  const [y, setY] = useState<TradeCoinType>();
  const [slippage, setSlippage] = useState(0.005);
  const [customSlippage, setCustomSlippage] = useState('');
  const [openSwapModal, setOpenSwapModal] = useState(false);
  const [price, setPrice] = useState('');

  return (
    <>
      <DashboardContent maxWidth="xl">
        <Stack flexDirection="row" justifyContent="space-between">
          <Typography variant="h4">Swap</Typography>
        </Stack>
        <span className="mt-2">Trade tokens in an instant.</span>
        <SelectTokenPair
          key="to"
          onLoading={() => {}}
          onCallback={(x, y) => {
            setX(x);
            setY(y);
            const ratio = BigNumber(y!.amount).div(x!.amount);
            const fixedRatio = ratio.toFixed(8, 1);
            const finalRatio = ratio.isInteger() ? ratio.toFixed(0) : fixedRatio;
            setPrice(formatByIntl(finalRatio));
          }}
        />
        <span className="text-gray-400 text-sm mt-4">
          Price {x && y ? `1 ${x?.symbol} ≈ ${price} ${y?.symbol}` : '-'}
        </span>
        <span className="mt-4">Slippage</span>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mt: 2 }}>
          <Box>
            {[0.005, 0.01, 0.03].map((item, index) => (
              <Button
                key={item.toString()}
                variant={slippage === item ? 'contained' : 'outlined'}
                size="small"
                sx={{ mr: 1 }}
                onClick={() => {
                  if (slippage === item) {
                    setSlippage(0);
                  } else {
                    setSlippage(item);
                    setCustomSlippage('');
                  }
                }}
              >
                {item * 100}%
              </Button>
            ))}
            <FormControl>
              <TextField
                sx={{
                  width: '90px',
                  height: '30px',
                  '& .MuiInputBase-root': {
                    height: '30px',
                    fontSize: '0.875rem',
                  },
                }}
                placeholder="0"
                id="outlined-basic"
                value={customSlippage}
                variant="outlined"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                  const { value } = e.target;
                  if (!isNumber(value)) {
                    return;
                  }
                  const numberValue = Number(value);

                  setSlippage(0);
                  setCustomSlippage(numberValue > 100 ? '100' : numberValue < 0 ? '0' : value);
                }}
              />
            </FormControl>
            <span className="text-gray-400 text-sm ml-1">%</span>
          </Box>

          <Button
            variant="contained"
            onClick={() => {
              setOpenSwapModal(true);
            }}
            disabled={!x || !y}
            sx={{ width: { xs: '100%', sm: 'fit-content' } }}
          >
            Swap
          </Button>
        </Box>
      </DashboardContent>
      {x && y && (
        <SwapConfirmModal
          slippage={
            slippage === 0
              ? customSlippage === '' || customSlippage === '0'
                ? 0
                : Number(customSlippage) / 100
              : slippage
          }
          open={openSwapModal}
          onClose={() => setOpenSwapModal(false)}
          x={x!}
          y={y!}
        />
      )}
    </>
  );
}
