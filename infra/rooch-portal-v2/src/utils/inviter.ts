import type { NetworkType, ThirdPartyAddress } from '@roochnetwork/rooch-sdk';

export const INVITER_ADDRESS_KEY = 'inviter-address';

export const getTwitterShareText = (network: NetworkType, address?: ThirdPartyAddress) => {
  const networkText = network === 'mainnet' ? 'PreMainnet' : 'Testnet';
  return `BTC:${address?.toStr()}

Rooch ${networkText} is live! Bind your Twitter to earn  RGas, and visit https://${network === 'mainnet' ? '' : 'test-'}grow.rooch.network to earn rewards with your BTC.

Join Rooch:
${getShareLink(network, address)}

#RoochNetwork #${networkText}`;
};

export const getShareLink = (network: NetworkType, address?: ThirdPartyAddress) =>
  `https://${network === 'mainnet' ? '' : 'test-'}portal.rooch.network/inviter/${address?.toStr()}`;
