import type { IndexerStateIDView } from '@roochnetwork/rooch-sdk';

import { useRef, useMemo, useState } from 'react';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Skeleton, CardHeader, Typography, CardContent } from '@mui/material';

import { EmptyContent } from 'src/components/empty-content/empty-content';

import { shortAddress } from '../../../utils/address';

export default function BBLlList({ address }: { address: string }) {
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 });
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateIDView | null }>({});

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) return;

    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    });
  };

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1] || null,
      pageSize: paginationModel.pageSize.toString(),
    }),
    [paginationModel]
  );

  const { data: bbns, isPending: isQueryBBNsPending } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type_with_owner: {
        owner: address,
        object_type: '0x4::bbn::BBNStakeSeal',
      },
    },
    queryOption: {
      decode: true,
      showDisplay: true,
    },
    cursor: queryOptions.cursor as IndexerStateIDView | null,
    limit: queryOptions.pageSize,
  });

  return (
    <Card>
      <CardHeader title="Babylon Staking" sx={{ mb: 1 }} />
      <CardContent
        className="!pt-2"
        component={Box}
        gap={3}
        display="grid"
        gridTemplateColumns={
          bbns?.data.length === 0
            ? undefined
            : {
                xs: 'repeat(1, 1fr)',
                sm: 'repeat(2, 1fr)',
                md: 'repeat(3, 1fr)',
                lg: 'repeat(4, 1fr)',
              }
        }
      >
        {isQueryBBNsPending ? (
          Array.from({ length: 4 }).map((i, index) => <Skeleton key={index} height={256} />)
        ) : bbns?.data.length === 0 ? (
          <EmptyContent title="No Babylon Staking Found" sx={{ py: 3 }} />
        ) : (
          bbns?.data.map((i) => (
            <Card key={i.id} elevation={0} className="!bg-gray-100 !shadow-none">
              <CardHeader
                title={
                  <span>
                    Stake ID <span className="text-sm">{shortAddress(i.id, 6, 4)}</span>
                  </span>
                }
              />
              <CardContent>
                <Typography
                  noWrap
                  sx={{
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                    overflowWrap: 'break-word',
                  }}
                >
                  {i.decoded_value?.value.staking_value as string} Sats
                </Typography>
              </CardContent>
            </Card>
          ))
        )}
      </CardContent>
    </Card>
  );
}
