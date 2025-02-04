import BigNumber from 'bignumber.js';

import { Stack, Button, TextField, FormControl, InputAdornment } from '@mui/material';

import { isNumber } from 'src/utils/reg';
import { formatByIntl } from 'src/utils/number';

interface TokenPairProps {
  max: number;
  amount: string;
  onChange: (amount: string) => void;
}

export default function AmountInput({ max, amount, onChange }: TokenPairProps) {
  return (
    <FormControl>
      <TextField
        label="Amount"
        placeholder=""
        value={amount}
        inputMode="decimal"
        autoComplete="off"
        onChange={(e) => {
          const value = e.target.value.replaceAll(',', '');
          if (!isNumber(value)) {
            return;
          }

          const nValue = Number(value) > max ? max : Number(value);
          onChange(formatByIntl(nValue));
        }}
        InputProps={{
          endAdornment: (
            <InputAdornment position="end">
              <Stack direction="row" spacing={0.5}>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => {
                    onChange(formatByIntl(new BigNumber(max).div(2).toString()));
                    // onChange();
                  }}
                >
                  Half
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => {
                    onChange(formatByIntl(max));
                  }}
                >
                  Max
                </Button>
              </Stack>
            </InputAdornment>
          ),
        }}
      />
    </FormControl>
  );
}
