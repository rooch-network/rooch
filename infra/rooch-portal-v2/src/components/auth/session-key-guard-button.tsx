import type { ReactNode } from 'react';

import { useState, useEffect } from 'react';
import {
  useRemoveSession,
  useCurrentSession,
  useCreateSessionKey,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';

import { isSessionExpired } from 'src/utils/common';

import { toast } from 'src/components/snackbar';

export default function SessionKeyGuardButton({ children }: { children: ReactNode }) {
  const sessionKey = useCurrentSession();
  const { mutateAsync: createSessionKey } = useCreateSessionKey();
  const { mutateAsync: removeSessionKey } = useRemoveSession();
  const [sessionKeyLoading, setSessionKeyLoading] = useState(false);

  const [isCurrentSessionExpired, setIsCurrentSessionExpired] = useState(false);

  useEffect(() => {
    if (sessionKey) {
      const sessionKeyJson = sessionKey.toJSON();
      const { lastActiveTime, maxInactiveInterval } = sessionKeyJson;
      if (isSessionExpired(Number(lastActiveTime), Number(maxInactiveInterval))) {
        setIsCurrentSessionExpired(true);
      }
    }
  }, [sessionKey]);

  return sessionKey && !isCurrentSessionExpired ? (
    children
  ) : (
    <LoadingButton
      fullWidth
      variant={sessionKey ? 'soft' : 'contained'}
      color={sessionKey ? 'error' : 'primary'}
      loading={sessionKeyLoading}
      onClick={async () => {
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
      Create Session Key
    </LoadingButton>
  );
}
