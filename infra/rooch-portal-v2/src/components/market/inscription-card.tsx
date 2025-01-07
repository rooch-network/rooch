import Link from 'next/link';

import { yellow } from '@mui/material/colors';
import { Chip, Stack, Typography } from '@mui/material';

import { fNumber } from 'src/utils/format-number';

import { Iconify } from 'src/components/iconify';

export interface InscriptionCardProps {
  objectId?: string;
  tick: string;
  isVerified: boolean;
  tokenBalance: string;
  selectMode?: boolean;
}

export default function InscriptionCard({
  objectId,
  tick,
  isVerified,
  tokenBalance,
  selectMode,
}: InscriptionCardProps) {
  return (
    <Stack
      justifyContent="center"
      alignItems="center"
      spacing={1}
      sx={{
        p: 1,
        borderRadius: '4px',
      }}
    >
      <Stack
        sx={{
          width: '100%',
        }}
        direction="row"
        justifyContent="space-between"
      >
        <Chip
          size="small"
          label={
            <Stack direction="row" alignItems="center">
              {tick}
              {isVerified && (
                <Iconify
                  icon="solar:verified-check-bold"
                  color={yellow.A200}
                  width={16}
                  sx={{
                    ml: 0.5,
                  }}
                />
              )}
            </Stack>
          }
          variant="filled"
          color="secondary"
        />
        {objectId && (
          <>
            {selectMode ? (
              <Chip size="small" label={`#${objectId.slice(2, 8)}`} variant="soft" color="info" />
            ) : (
                <Chip size="small" label={`#${objectId.slice(2, 8)}`} variant="soft" color="info" />
            )}
          </>
        )}
      </Stack>
      <Typography
        className="text-gray-400"
        sx={{
          fontSize: '0.875rem',
          fontWeight: 600,
        }}
      >
        Token Balance
      </Typography>
      <Typography
        sx={{
          fontSize: '2rem',
          fontWeight: 600,
        }}
      >
        {fNumber(tokenBalance)}
      </Typography>
    </Stack>
  );
}
