import type { StackProps } from '@mui/material/Stack';

import { useState } from 'react';
import {
  useRemoveSession,
  useCurrentSession,
  SessionKeyGuard,
} from '@roochnetwork/rooch-sdk-kit';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import { LoadingButton } from '@mui/lab';
import Typography from '@mui/material/Typography';

export function NavUpgrade({ sx, ...other }: StackProps) {
  const { mutateAsync: removeSessionKey } = useRemoveSession();
  const sessionKey = useCurrentSession();

  const [sessionKeyLoading, setSessionKeyLoading] = useState(false);

  return (
    <Stack sx={{ px: 2, py: 5, textAlign: 'center', ...sx }} {...other}>
      <Stack alignItems="center">
        <Box sx={{ position: 'relative' }} />

        <Stack spacing={0.5} sx={{ mb: 2, mt: 1.5, width: 1 }}>
          <Typography
            variant="subtitle2"
            noWrap
            sx={{ color: 'var(--layout-nav-text-primary-color)' }}
          >
            Rooch
          </Typography>

          <Typography
            variant="body2"
            noWrap
            sx={{ color: 'var(--layout-nav-text-disabled-color)' }}
          >
            Rooch Portal
          </Typography>
        </Stack>

        <SessionKeyGuard onClick={() => {
          if (sessionKey) {
            setSessionKeyLoading(true)
            removeSessionKey({authKey: sessionKey.getAuthKey()}).then((_) => {
              setSessionKeyLoading(false)
            })
          }
        }}>
        <LoadingButton
          variant={sessionKey ? 'soft' : 'contained'}
          color={sessionKey ? 'error' : 'primary'}
          loading={sessionKeyLoading}
        >
          {sessionKey ? 'Clear Session Key' : 'Create Session Key'}
        </LoadingButton>
      </SessionKeyGuard>
    </Stack>
    </Stack >
  );
}
