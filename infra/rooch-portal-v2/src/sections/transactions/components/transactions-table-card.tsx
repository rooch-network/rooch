import type { PaginatedTransactionWithInfoViews } from '@roochnetwork/rooch-sdk';

import dayjs from 'dayjs';

import {
  Box,
  Card,
  Chip,
  Table,
  Stack,
  Button,
  Tooltip,
  TableRow,
  TableBody,
  TableCell,
  CardHeader,
  Typography,
  Pagination,
} from '@mui/material';

import { RouterLink } from 'src/routes/components';

import { shortAddress } from 'src/utils/address';
import { formatCoin } from 'src/utils/format-number';
import { getUTCOffset } from 'src/utils/format-time';

import { ROOCH_GAS_COIN_DECIMALS } from 'src/config/constant';

import { Scrollbar } from 'src/components/scrollbar';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import { TRANSACTION_TYPE_MAP, TRANSACTION_STATUS_TYPE_MAP } from '../constant';

export default function TransactionsTableCard({
  address,
  isPending,
  transactionsList,
  paginationModel,
  paginate,
  dense,
}: {
  address: string;
  isPending: boolean;
  transactionsList?: PaginatedTransactionWithInfoViews;
  paginationModel?: {
    index: number;
    limit: number;
  };
  paginate?: (index: number) => void;
  dense?: boolean;
}) {
  return (
    <Card className="mt-4">
      <CardHeader
        title="Transactions"
        subheader={dense ? undefined : `${shortAddress(address, 6, 6)} Activity History`}
        sx={{ mb: 3 }}
      />
      <Scrollbar sx={{ minHeight: dense ? undefined : 462 }}>
        <Table sx={{ minWidth: 720 }} size={dense ? 'small' : 'medium'}>
          <TableHeadCustom
            headLabel={[
              { id: 'coin', label: 'Transaction Hash' },
              {
                id: 'timestamp',
                label: (
                  <Box>
                    Timestamp <span className="text-xs ml-1">({getUTCOffset()})</span>
                  </Box>
                ),
              },
              { id: 'status', label: 'Status' },
              { id: 'type', label: 'Type' },
              { id: 'gas', label: 'Gas' },
              { id: 'action', label: 'Action', align: 'center' },
            ]}
          />
          <TableBody>
            {isPending ? (
              <TableSkeleton col={6} row={dense ? 5 : 10} rowHeight="69px" />
            ) : (
              <>
                {transactionsList?.data.map((item) => (
                  <TableRow key={item.execution_info?.tx_hash}>
                    <TableCell width="256px">
                      <Typography className="!font-mono !font-medium">
                        <Tooltip title={item.execution_info?.tx_hash} arrow>
                          <span>{shortAddress(item.execution_info?.tx_hash, 8, 6)}</span>
                        </Tooltip>
                      </Typography>
                    </TableCell>
                    <TableCell>
                      {dayjs(Number(item.transaction.sequence_info.tx_timestamp)).format(
                        'MMMM DD, YYYY HH:mm:ss'
                      )}
                    </TableCell>
                    {item.execution_info && (
                      <TableCell>
                        <Chip
                          label={TRANSACTION_STATUS_TYPE_MAP[item.execution_info.status.type].text}
                          size="small"
                          variant="soft"
                          color={TRANSACTION_STATUS_TYPE_MAP[item.execution_info.status.type].color}
                        />
                      </TableCell>
                    )}
                    <TableCell>
                      <Chip
                        label={TRANSACTION_TYPE_MAP[item.transaction.data.type].text}
                        size="small"
                        variant="soft"
                        color={TRANSACTION_TYPE_MAP[item.transaction.data.type].color}
                      />
                    </TableCell>
                    {item.execution_info && (
                      <TableCell className="!text-xs">
                        {formatCoin(
                          Number(item.execution_info.gas_used),
                          ROOCH_GAS_COIN_DECIMALS,
                          6
                        )}
                      </TableCell>
                    )}
                    {item.execution_info && (
                      <TableCell align="center">
                        <Button component={RouterLink} href={`/tx/${item.execution_info.tx_hash}`}>
                          View
                        </Button>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
                <TableNoData
                  title="No Transaction Found"
                  notFound={transactionsList?.data.length === 0}
                />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
      {!dense && paginationModel && paginate && (
        <Stack className="mb-4 w-full mt-4" alignItems="flex-end">
          <Pagination
            count={
              transactionsList?.has_next_page ? paginationModel.index + 1 : paginationModel.index
            }
            page={paginationModel.index}
            onChange={(event: React.ChangeEvent<unknown>, value: number) => {
              paginate(value);
            }}
          />
        </Stack>
      )}
      {dense && (transactionsList?.data.length || 0) > 0 && (
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
