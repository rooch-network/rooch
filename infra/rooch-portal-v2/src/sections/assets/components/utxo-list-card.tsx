import type { IndexerStateIDView } from '@roochnetwork/rooch-sdk';

import { useRef, useMemo, useState } from 'react';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Skeleton, CardHeader, CardContent } from '@mui/material';

import { shortAddress } from 'src/utils/address';
import { fNumber } from 'src/utils/format-number';

import { EmptyContent } from 'src/components/empty-content';

export default function UTXOList({ address }: { address: string }) {
  const [selectedUTXO, setSelectUTXO] = useState('');
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 });
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateIDView | null }>({});

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1] || undefined,
      pageSize: paginationModel.pageSize.toString(),
    }),
    [paginationModel]
  );

  const { data: utxoList, isPending: isUTXOPending } = useRoochClientQuery(
    'queryUTXO',
    {
      filter: {
        owner: address,
      },
      cursor: queryOptions.cursor,
      limit: queryOptions.pageSize,
    },
    { enabled: !!address }
  );

  return (
    <Card>
      <CardHeader title="BTC" sx={{ mb: 1 }} />
      <CardContent
        className="!pt-2"
        component={Box}
        gap={3}
        display="grid"
        gridTemplateColumns={
          utxoList?.data.length === 0
            ? undefined
            : {
                xs: 'repeat(1, 1fr)',
                sm: 'repeat(2, 1fr)',
                md: 'repeat(3, 1fr)',
                lg: 'repeat(4, 1fr)',
              }
        }
      >
        {isUTXOPending ? (
          Array.from({ length: 4 }).map((i, index) => <Skeleton key={index} height={124} />)
        ) : utxoList?.data.length === 0 ? (
          <EmptyContent title="No BTC UTXOs Found" sx={{ py: 3 }} />
        ) : (
          utxoList?.data.map((i) => (
            <Card key={i.id} elevation={0} className="!bg-gray-100 !shadow-none">
              <CardHeader
                title={
                  <span>
                    UTXO <span className="text-sm">{shortAddress(i.id, 6, 4)}</span>
                  </span>
                }
              />
              <CardContent>{fNumber(i.value?.value)} Sats</CardContent>
            </Card>
          ))
        )}
      </CardContent>
    </Card>
  );
}
