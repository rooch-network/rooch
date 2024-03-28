import { WalletsMaterialProps } from './interface'

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
