// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { WalletsMaterialProps } from './interface'

export const ROOCH_OPERATING_ADDRESS = import.meta.env.VITE_ROOCH_OPERATING_ADDRESS

// ** Wallet Connect
export const walletsMaterialMap = new Map<string, WalletsMaterialProps>([
  [
    'unisat',
    {
      name: 'Unisat',
      icon: '/icon-unisat.svg',
      description: 'Unisat wallet',
      types: ['btc'],
      link: 'https://chromewebstore.google.com/detail/unisat-wallet/ppbibelpcjmhbdihakflkdcoccbgbkpo',
    },
  ],
  [
    'metamask',
    {
      name: 'MetaMask',
      icon: '/icon-metamask.svg',
      description: 'Metmask wallet',
      types: ['eth', 'bsc'],
      link: 'https://chromewebstore.google.com/detail/metamask/nkbihfbeogaeaoehlefnkodbefgpgknn',
    },
  ],
  [
    'okx',
    {
      name: 'OKX',
      icon: '/icon-okx.svg',
      description: 'OKX wallet',
      types: ['btc'],
      link: 'https://chromewebstore.google.com/detail/okx-wallet/mcohilncbfahbmgdjkbpemcciiolgcge',
    },
  ],
])

// ** Assets NFT
export const nftData = [
  {
    id: 1,
    imageUrl:
      'https://i.seadn.io/s/raw/files/96f26dfaeb80982b4c48ef7b6d1a42a1.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 2,
    imageUrl:
      'https://i.seadn.io/s/raw/files/7700594825d9090b03f7134a9f96d9f0.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 3,
    imageUrl:
      'https://i.seadn.io/s/raw/files/d0f989ab16333bbf348fc74f0d4a6d8d.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 4,
    imageUrl:
      'https://i.seadn.io/s/raw/files/c8edb3d3eb5549a10f3cd2a919c7e6e6.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 5,
    imageUrl:
      'https://i.seadn.io/s/raw/files/96f26dfaeb80982b4c48ef7b6d1a42a1.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 6,
    imageUrl:
      'https://i.seadn.io/s/raw/files/7700594825d9090b03f7134a9f96d9f0.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 7,
    imageUrl:
      'https://i.seadn.io/s/raw/files/d0f989ab16333bbf348fc74f0d4a6d8d.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
  {
    id: 8,
    imageUrl:
      'https://i.seadn.io/s/raw/files/c8edb3d3eb5549a10f3cd2a919c7e6e6.png?auto=format&dpr=1&w=640',
    title: 'Rooch OG NFT',
    price: '6.988 ETH',
  },
]
