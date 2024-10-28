import type { StackProps } from '@mui/material/Stack';

import { useState } from 'react';
import {
  useRemoveSession,
  useCurrentSession,
  useCreateSessionKey,
} from '@roochnetwork/rooch-sdk-kit';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import { LoadingButton } from '@mui/lab';
import Typography from '@mui/material/Typography';

import { toast } from 'src/components/snackbar';

export function NavUpgrade({ sx, ...other }: StackProps) {
  const { mutateAsync: createSessionKey } = useCreateSessionKey();
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

        <LoadingButton
          variant={sessionKey ? 'soft' : 'contained'}
          color={sessionKey ? 'error' : 'primary'}
          loading={sessionKeyLoading}
          onClick={async () => {
            if (sessionKey) {
              await removeSessionKey({ authKey: sessionKey.getAuthKey() });
              return;
            }
            try {
              setSessionKeyLoading(true);
              await createSessionKey({
                appName: 'rooch-portal',
                appUrl: 'portal.rooch.network',
                scopes: [
                  '0x1::*::*',
                  '0x3::*::*',
                  '0x176214bed3764a1c6a43dc1add387be5578ff8dbc263369f5bdc33a885a501ae::*::*',
                  '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3::*::*',
                ],
                maxInactiveInterval: 60 * 60 * 8,
              });
            } catch (error) {
              if (error.message) {
                toast.error(error.message);
                return;
              }
              toast.error(String(error));
            } finally {
              setSessionKeyLoading(false);
            }
          }}
        >
          {sessionKey ? 'Clear Session Key' : 'Create Session Key'}
        </LoadingButton>
      </Stack>
    </Stack>
  );
}
