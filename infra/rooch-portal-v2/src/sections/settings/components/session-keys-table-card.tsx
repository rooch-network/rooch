import type { SessionInfoView, PaginationResult } from '@roochnetwork/rooch-sdk';
import type { RefetchOptions, QueryObserverResult } from '@tanstack/react-query';

import { useCallback } from 'react';
import { useRemoveSession } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Table, Stack, Button, TableBody, CardHeader, Pagination } from '@mui/material';

import { RouterLink } from 'src/routes/components';

import { getUTCOffset } from 'src/utils/format-time';

import { Scrollbar } from 'src/components/scrollbar';
import { TableHeadCustom } from 'src/components/table';
import TableSkeleton from 'src/components/skeleton/table-skeleton';

import SessionKeyRowItem from './session-key-row-item';

export default function SessionKeysTableCard({
  address,
  isPending,
  refetchSessionKeys,
  sessionKeys,
  paginationModel,
  paginate,
  dense,
}: {
  address: string;
  isPending: boolean;
  refetchSessionKeys: (
    options?: RefetchOptions
  ) => Promise<QueryObserverResult<PaginationResult<string, SessionInfoView>, Error>>;
  sessionKeys?: PaginationResult<string, SessionInfoView>;
  paginationModel?: {
    index: number;
    limit: number;
  };
  paginate?: (index: number) => void;
  dense?: boolean;
}) {
  const { mutateAsync: removeSession } = useRemoveSession();

  const remove = useCallback(
    async (authKey: string) => {
      await removeSession({ authKey });
      await refetchSessionKeys();
    },
    [removeSession, refetchSessionKeys]
  );

  return (
    <Card className="mt-4">
      <CardHeader
        title="Session Keys"
        subheader="Manage the site that your account has authorized session keys"
        sx={{ mb: 3 }}
      />
      <Scrollbar sx={{ minHeight: dense ? undefined : 462 }}>
        <Table sx={{ minWidth: 720 }} size={dense ? 'small' : 'medium'}>
          <TableHeadCustom
            headLabel={[
              { id: 'app', label: 'App' },
              { id: 'scope', label: 'Scope' },
              {
                id: 'grantedAt',
                label: (
                  <Box>
                    Granted at <span className="text-xs ml-1">({getUTCOffset()})</span>
                  </Box>
                ),
              },
              {
                id: 'lastActiveAt',
                label: (
                  <Box>
                    Last Active at <span className="text-xs ml-1">({getUTCOffset()})</span>
                  </Box>
                ),
              },
              { id: 'expirationInterval', label: 'Expiration Interval(sec.)', align: 'center' },
              { id: 'action', label: 'Action', align: 'center' },
            ]}
          />
          <TableBody>
            {isPending ? (
              <TableSkeleton col={6} row={dense ? 5 : 10} rowHeight="69px" />
            ) : (
              sessionKeys?.data.map((item) => (
                <SessionKeyRowItem
                  key={item.authenticationKey}
                  item={item}
                  removeSession={remove}
                />
              ))
            )}
          </TableBody>
        </Table>
      </Scrollbar>
      {!dense && paginationModel && paginate && (
        <Stack className="mb-4 w-full" alignItems="flex-end">
          <Pagination
            count={sessionKeys?.hasNextPage ? paginationModel.index + 1 : paginationModel.index}
            page={paginationModel.index}
            onChange={(event: React.ChangeEvent<unknown>, value: number) => {
              paginate(value);
            }}
          />
        </Stack>
      )}
      {dense && (
        <Stack alignItems="center" className="my-2">
          <Button
            variant="text"
            color="primary"
            component={RouterLink}
            href={`/transactions/${address}`}
          >
            View All
          </Button>
        </Stack>
      )}
    </Card>
  );
}
