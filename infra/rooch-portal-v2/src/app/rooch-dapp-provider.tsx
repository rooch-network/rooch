'use client';

import type { ReactNode } from 'react';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RoochProvider, WalletProvider } from '@roochnetwork/rooch-sdk-kit';

import { networkConfig } from 'src/hooks/use-networks';

const queryClient = new QueryClient();

export default function RoochDappProvider({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <RoochProvider networks={networkConfig} defaultNetwork="mainnet">
        <WalletProvider preferredWallets={['UniSat']} chain="bitcoin" autoConnect>
          {children}
        </WalletProvider>
      </RoochProvider>
    </QueryClientProvider>
  );
}
