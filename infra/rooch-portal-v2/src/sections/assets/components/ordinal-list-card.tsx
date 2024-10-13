import type { ReactNode } from 'react';
import type { IndexerStateIDView } from '@roochnetwork/rooch-sdk';

import DOMPurify from 'dompurify';
import { useRef, useMemo, useState } from 'react';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Chip, Skeleton, CardHeader, Typography, CardContent } from '@mui/material';

import { hexToString } from 'src/utils/common';

import { EmptyContent } from 'src/components/empty-content/empty-content';

export default function OrdinalList({ address }: { address: string }) {
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 });
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateIDView | null }>({});

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

  const { data: ordinalList, isPending: isOrdinalListPending } = useRoochClientQuery(
    'queryInscriptions',
    {
      filter: {
        owner: address,
      },
      cursor: queryOptions.cursor as IndexerStateIDView | null,
      limit: queryOptions.pageSize,
    }
  );

  return (
    <Card>
      <CardHeader title="Ordinals" sx={{ mb: 1 }} />
      <CardContent
        className="!pt-2"
        component={Box}
        gap={3}
        display="grid"
        gridTemplateColumns={
          ordinalList?.data.length === 0
            ? undefined
            : {
                xs: 'repeat(1, 1fr)',
                sm: 'repeat(2, 1fr)',
                md: 'repeat(3, 1fr)',
                lg: 'repeat(4, 1fr)',
              }
        }
      >
        {isOrdinalListPending ? (
          Array.from({ length: 4 }).map((i, index) => <Skeleton key={index} height={256} />)
        ) : ordinalList?.data.length === 0 ? (
          <EmptyContent title="No Ordinals Found" sx={{ py: 3 }} />
        ) : (
          ordinalList?.data.map((i) => {
            let parsePlainText: string | null | undefined | ReactNode = '';
            try {
              parsePlainText =
                i.value.content_type === 'text/plain;charset=utf-8'
                  ? DOMPurify.sanitize(JSON.stringify(
                      JSON.parse(hexToString(i.value.body as unknown as string)),
                      null,
                      2
                    ))
                  : i.value.content_type;
            } catch (error) {
              parsePlainText = (
                <Chip label="Parse Error" size="small" variant="soft" color="error" />
              );
            }
            return (
              <Card key={i.id} elevation={0} className="!bg-gray-100 !shadow-none">
                <CardHeader title={i.value.inscription_number} subheader="Inscriptions #" />
                <CardContent>
                  <Typography
                    noWrap
                    sx={{
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-word',
                      overflowWrap: 'break-word',
                    }}
                  >
                    {parsePlainText}
                  </Typography>
                </CardContent>
              </Card>
            );
          })
        )}
      </CardContent>
    </Card>
  );
}
