import { useEffect } from 'react';
import { useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import { useRouter } from 'src/routes/hooks';

export default function useAddressChanged({ address, path }: { address: string; path: string }) {
  const router = useRouter();
  const currentAddress = useCurrentAddress();
  useEffect(() => {
    if (currentAddress && currentAddress.toStr() !== address) {
      router.push(`/${path}/${currentAddress.toStr()}`);
    }
  }, [currentAddress, path, address, router]);
}
