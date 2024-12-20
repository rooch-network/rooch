'use client';

import type { ReactNode } from 'react';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RoochProvider, WalletProvider } from '@roochnetwork/rooch-sdk-kit';

import { networkConfig } from 'src/hooks/use-networks';

import { isMainNetwork } from '../utils/env'

const queryClient = new QueryClient();

export default function RoochDappProvider({ children }: { children: ReactNode }) {
  const network = isMainNetwork() ? 'mainnet' : 'testnet'
  return (
    <QueryClientProvider client={queryClient}>
      <RoochProvider networks={networkConfig} defaultNetwork={network} sessionConf={
        {
          appName: 'rooch-portal',
          appUrl: 'portal.rooch.network',
          scopes: [
            '0x1::*::*',
            '0x3::*::*',
            '0x176214bed3764a1c6a43dc1add387be5578ff8dbc263369f5bdc33a885a501ae::*::*',
            '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3::*::*',
          ],
          maxInactiveInterval: 60 * 60 * 8,
        }
      }>
        <WalletProvider chain="bitcoin" autoConnect>
          {children}
        </WalletProvider>
      </RoochProvider>
    </QueryClientProvider>
  );
}
