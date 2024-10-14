'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

export default function TransactionOverviewView() {
  const address = useCurrentAddress();
  const router = useRouter();
  useEffect(() => {
    if (address) {
      router.push(`/transactions/${address.toStr()}`);
    }
  }, [address, router]);
  if (!address) {
    return null;
  }
  return null;
}
