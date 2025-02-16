import { darken, ToggleButton, ToggleButtonGroup } from '@mui/material';

import type { PoolVersion } from './types';

export interface PoolVersionSelectProps {
  version?: number;
  onChange?: (version: PoolVersion) => void;
}

export default function PoolVersionSelect({ version, onChange }: PoolVersionSelectProps) {
  return (
    <ToggleButtonGroup
      exclusive
      value={version}
      onChange={(_, value) => {
        if (value !== null && onChange) {
          onChange(value);
        }
      }}
      sx={{
        '& .MuiToggleButton-root': {
          fontSize: '0.875rem',
          fontWeight: 600,
          lineHeight: '100%',
          color: '#9DA3B1',
          border: 'none',
          background: '#DCDEE1',
          width: '50px',
          transition: 'all 0.3s ease',
          padding: '8.55px 0',
          '&.Mui-selected': {
            background: '#EFF0F0',
            color: '#101828',
            '&:hover': {
              background: darken('#EFF0F0', 0.02),
            },
          },
          '&:hover': {
            background: darken('#DCDEE1', 0.1),
          },
        },
        '& .MuiToggleButtonGroup-grouped:not(:last-of-type)': {
          borderRight: '2px solid white',
        },
      }}
    >
      <ToggleButton value={0}>V0</ToggleButton>
      <ToggleButton value={0.5}>V0.5</ToggleButton>
    </ToggleButtonGroup>
  );
}
