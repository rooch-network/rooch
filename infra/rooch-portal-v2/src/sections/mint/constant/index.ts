const FMNFT_ADDRESS = '0x176214bed3764a1c6a43dc1add387be5578ff8dbc263369f5bdc33a885a501ae';
export const FMNFT = {
  type: 'nft',
  objType: `${FMNFT_ADDRESS}::og_nft::NFT`,
  action: `/mint/free/nft/${FMNFT_ADDRESS}`,
  name: 'Rooch Pioneer',
  symbol: 'FMNFT',
  distribution: 'Free Mint',
  progress: -1,
  data: {
    address: FMNFT_ADDRESS,
  },
};
