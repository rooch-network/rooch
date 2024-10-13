import { useState, useEffect } from 'react';
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit';

import { Card, Table, TableBody } from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { toast } from 'src/components/snackbar';
import { Scrollbar } from 'src/components/scrollbar';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import { FMNFT } from '../constant';
import MintRowItem from './mint-row-item';
import { getTokenInfo } from '../utils/get-token-info';

import type { TokenInfo } from '../utils/get-token-info';

const COMING_SOON_TOKENS: MintType[] = [
  {
    symbol: 'HDCL',
    name: 'BTC Holder Coin (with Lock)',
    distribution: 'Self-Staking (with Lock)',
    progress: 0,
    type: 'nft',
    action: '',
    data: {
      address: '',
    },
  },
  {
    symbol: 'FMC',
    name: 'Free Mint Coin',
    distribution: 'Free Mint',
    progress: 0,
    type: 'nft',
    action: '',
    data: {
      address: '',
    },
  },
  {
    symbol: 'fmSFT',
    name: 'Free Mint SFT',
    distribution: 'Free Mint',
    progress: 0,
    type: 'nft',
    action: '',
    data: {
      address: '',
    },
  },
  {
    symbol: 'EBC',
    name: 'Epoch Bus Coin',
    distribution: 'Epoch Bus',
    progress: 0,
    type: 'nft',
    action: '',
    data: {
      address: '',
    },
  },
  {
    symbol: 'HMC',
    name: 'Hardware Mining Coin',
    distribution: 'Hardware Mining',
    progress: 0,
    type: 'nft',
    action: '',
    data: {
      address: '',
    },
  },
  {
    symbol: 'BEC',
    name: 'Burn to Earn Coin',
    distribution: 'Burn to Earn',
    progress: 0,
    type: 'nft',
    action: '',
    data: {
      address: '',
    },
  },
];
type NFTInfo = {
  address: string;
};

export type MintType = {
  type: 'nft' | 'self_staking';
  name: string;
  symbol: string;
  distribution: string;
  progress: number;
  action: string;
  data: NFTInfo | TokenInfo;
};

export default function MintTableCard({
  dense,
  isStaticData,
}: {
  dense?: boolean;
  isStaticData?: boolean;
}) {
  const client = useRoochClient();
  const addresses = useNetworkVariable('mintAddress');

  const [tokenList, setTokenList] = useState<MintType[]>(isStaticData ? COMING_SOON_TOKENS : []);

  const [isLoading, setIsLoading] = useState(!isStaticData);

  useEffect(() => {
    if (isStaticData || tokenList.length !== 0) {
      return;
    }
    const fetchTokenInfo = async () => {
      const data: MintType[] = [
        {
          ...FMNFT,
        } as MintType,
      ];

      const tokenPromises = addresses.map(async (item) => {
        try {
          const token = await getTokenInfo(client, item);
          if (token) {
            return {
              type: 'self_staking',
              action: `/mint/self/staking/${token.address}`,
              name: token.coin.name,
              symbol: token.coin.symbol,
              distribution: 'Self-Staking (without Lock)',
              progress: token.progress,
              data: token,
            };
          }
        } catch (error) {
          toast.error(String(error));
        }
        return null;
      });

      const tokens = (await Promise.all(tokenPromises)).filter(Boolean) as MintType[];
      setIsLoading(false);

      setTokenList([...data, ...tokens.filter(Boolean)]);
    };

    fetchTokenInfo();
  }, [isStaticData, client, addresses, tokenList.length]);

  const tableHeaders: Record<string, any>[] = [
    { id: 'symbol', label: 'Symbol' },
    { id: 'name', label: 'Name' },
    { id: 'mechanism', label: 'Distribution Mechanism' },
    { id: 'progress', label: 'Progress' },
  ];

  if (isStaticData) {
    tableHeaders.push({ id: 'action', label: 'Action', align: 'right' });
  }

  return (
    <Card className="mt-4">
      <Scrollbar sx={{ minHeight: dense ? undefined : 462 }}>
        <Table sx={{ minWidth: 720 }} size={dense ? 'small' : 'medium'}>
          <TableHeadCustom headLabel={tableHeaders} />

          <TableBody>
            {isLoading ? (
              <TableSkeleton col={4} row={5} rowHeight="77px" />
            ) : (
              <>
                {tokenList.map((row) => (
                  <MintRowItem key={row.symbol} row={row} isStaticData={isStaticData} />
                ))}
                <TableNoData title="No Tokens" notFound={tokenList.length === 0} />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
    </Card>
  );
}
