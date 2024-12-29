export const NETWORK: 'mainnet' | 'testnet' = 'mainnet';

export const SUI_DECIMALS = 6;

export const NETWORK_PACKAGE = {
  mainnet: {
    MARKET_PACKAGE_ID: '0x5ce4eec53735abe180ebe12dab5d813d4443100241ea2b714a2fd76e5562a490',
    tickInfo: {
      grow: {
        MARKET_OBJECT_ID: '0x4dc9dde9dc7eabe0eb66913a09e8e47dc952771b9172824062d60670c91e35f6',
      },
      gold: {
        MARKET_OBJECT_ID: '0xf8a12cc79615988ef0f04d8542b18fe27d5f967972e30fd89328c37f5da9f288',
      },
    } as {
      [key: string]: {
        MARKET_OBJECT_ID: string;
      };
    },
  },
  testnet: {
    MARKET_PACKAGE_ID: '0x5ce4eec53735abe180ebe12dab5d813d4443100241ea2b714a2fd76e5562a490',
    tickInfo: {
      grow: {
        MARKET_OBJECT_ID: '0x4dc9dde9dc7eabe0eb66913a09e8e47dc952771b9172824062d60670c91e35f6',
      },
      gold: {
        MARKET_OBJECT_ID: '0xf8a12cc79615988ef0f04d8542b18fe27d5f967972e30fd89328c37f5da9f288',
      },
    } as {
      [key: string]: {
        MARKET_OBJECT_ID: string;
      };
    },
  },
};
