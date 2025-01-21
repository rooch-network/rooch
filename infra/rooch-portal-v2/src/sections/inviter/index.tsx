'use client';

import { useEffect } from 'react';

import { isValidAddress } from '@roochnetwork/rooch-sdk';
import { useRouter } from '../../routes/hooks';
import { INVITER_ADDRESS_KEY } from '../../utils/inviter';

export function InviterView({ inviterAddress }: { inviterAddress?: string }) {
  const router = useRouter();

  useEffect(() => {
    if (inviterAddress && isValidAddress(inviterAddress)) {
      window.localStorage.setItem(INVITER_ADDRESS_KEY, inviterAddress);
    }
    router.push('/settings');
  }, [inviterAddress, router]);

  return <></>;
}
