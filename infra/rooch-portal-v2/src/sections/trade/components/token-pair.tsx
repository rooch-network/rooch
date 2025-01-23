import type { SelectChangeEvent} from '@mui/material';

import { useState } from 'react';
import { useDebounce } from 'react-use';

import { Box, Button, Select , MenuItem , TextField ,  InputLabel, FormControl } from '@mui/material';

import type { TradeCoinType } from './types';

interface TokenPairProps {
  coins: TradeCoinType[];
  onChange: (coin: TradeCoinType, amount: number) => void;
}

export default function TokenPair({ coins, onChange }: TokenPairProps) {
  const [x, setX] = useState('');
  const [xValue, setXValue] = useState('');

  useDebounce(
    () => {
      const coin = coins.find((item) => item.symbol === x);
      onChange(coin!, Number(xValue));
    },
    500,
    []
  );

  return (
    <>
      <span className="mt-2">Choose the tokens you want to provide liquidity for.</span>
      <Box sx={{ display: 'flex', alignItems: 'center', mt: 4 }}>
        <FormControl sx={{ minWidth: 220 }}>
          <InputLabel id="select-x">X</InputLabel>
          <Select
            labelId="select-x"
            id="select"
            value={x}
            label="X"
            onChange={(e: SelectChangeEvent) => {
              console.log(e);
              setX(e.target.value);
            }}
          >
            {coins.map((item) => (
              <MenuItem id={item.symbol} value={`${item.type}&${item.balance}`}>
                <span>{item.symbol} :</span>
                <span> {item.balance}</span>
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl fullWidth sx={{ ml: 2 }}>
          <TextField
            id="outlined-basic"
            label="Amount"
            value={xValue}
            variant="outlined"
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
              setXValue(e.target.value);
            }}
          />
        </FormControl>
      </Box>
      <Box display="flex" alignItems="center" justifyContent="flex-end" sx={{ mt: 1 }}>
        <Box display="flex" alignItems="center">
          {[0.25, 0.5, 0.75, 1].map((item, index) => (
            <Button
              key={index}
              variant={item === 1 ? 'contained' : 'outlined'}
              size="small"
              sx={{ mx: 0.5 }}
              onClick={() => {
                setXValue((Number(x.split('&')[1]) * item).toString());
              }}
            >
              {item * 100}%
            </Button>
          ))}
        </Box>
      </Box>
      <span className="text-gray-400 text-sm mt-2">
        The amount earned providing liquidity. All pools have fixed 0.3% fees.
      </span>
    </>
  );
}
