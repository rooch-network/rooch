'use client';

import { useCountDown } from 'ahooks';
import dayjs from 'dayjs/esm/index.js';
import { useState, useCallback } from 'react';
import PuffLoader from 'react-spinners/PuffLoader';
import relativeTime from 'dayjs/esm/plugin/relativeTime';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { alpha } from '@mui/material/styles';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import {
  Tab,
  Card,
  Chip,
  Tabs,
  Stack,
  Table,
  Skeleton,
  TableRow,
  TableBody,
  TableCell,
  TableContainer,
} from '@mui/material';

import { grey, warning } from 'src/theme/core/palette';
import { ROOCH_MULTI_SIG_ADDRESS } from 'src/config/constant';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableHeadCustom } from 'src/components/table/table-head-custom';

import useTable from './components/use-table';
import TableNoData from './components/table-no-data';
import OrderTableRow from './components/order-table-row';
import { EVENT_ENUM, applyFilter, getComparator, ORDER_TYPE_MAP } from './utils/common';

import type { RenderEvent } from './utils/types';

dayjs.extend(relativeTime);

const STATUS_OPTIONS = [
  { value: 'All', label: 'All' },
  { value: 'Buy', label: 'Buy' },
  { value: 'Accept Bid', label: 'Accept Bid' },
  { value: 'List', label: 'List' },
  { value: 'Create Bid', label: 'Create Bid' },
  { value: 'Cancel List', label: 'Cancel List' },
];

const TABLE_HEAD = [
  { id: 'type', label: 'Event', width: 50 },
  { id: 'tick', label: 'Tick', width: 120 },
  { id: 'amount', label: 'Amount', width: 120 },
  { id: 'price', label: 'Total/Price', width: 150 },
  { id: 'sender', label: 'Sender', width: 120 },
  // { id: 'seller', label: 'Seller', width: 120 },
  // { id: 'buyer', label: 'Buyer', width: 120 },
  { id: 'time', label: 'Time', width: 120 },
];

export type activitiesFilter = 'All' | 'Buy' | 'List' | 'Cancel List' | 'Create Bid' | 'Accept Bid';

export const TICK_MAP: Record<
  string,
  {
    tick: string;
    types: string;
    fromTokenDecimals: bigint;
    fromTokenSymbol: string;
    toTokenDecimals: bigint;
    toTokenSymbol: string;
  }
> = {
  'rgas-grow': {
    tick: 'RGAS/GROW',
    types:
      '0x0000000000000000000000000000000000000000000000000000000000000003::gas_coin::RGas,0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3::grow_bitcoin::GROW',
    fromTokenDecimals: 8n,
    fromTokenSymbol: 'RGAS',
    toTokenDecimals: 0n,
    toTokenSymbol: 'GROW',
  },
  'rgas-gold': {
    tick: 'RGAS/GOLD',
    types:
      '0x0000000000000000000000000000000000000000000000000000000000000003::gas_coin::RGas,0xef98788d657de7354f9375a9e082343397d70b752dd59c97a065f6fe5e132152::gold::Gold',
    fromTokenDecimals: 8n,
    fromTokenSymbol: 'RGAS',
    toTokenDecimals: 6n,
    toTokenSymbol: 'GOLD',
  },
};

export default function MarketPlaceHistoryView({ tick }: { tick: string }) {
  const paramTick = tick.toLowerCase();

  const {
    data: historyList,
    isLoading,
    isFetching,
    refetch: refetchTransactionEvent,
  } = useRoochClientQuery('queryEvents', {
    filter: {
      event_type: `${ROOCH_MULTI_SIG_ADDRESS}::market_v2::OrderEvent<${TICK_MAP[paramTick].types}>`,
    },
  });
  console.log('🚀 ~ file: view.tsx:109 ~ MarketPlaceHistoryView ~ historyList:', historyList);

  const [filters, setFilters] = useState<{
    type: activitiesFilter;
  }>({
    type: 'All',
  });

  const tableData =
    (historyList?.data?.map((item) => ({
      ...item.decoded_event_data?.value,
      created_at: item.created_at,
      sender: item.sender,
      tx_hash: item.tx_hash,
    })) as RenderEvent[]) || [];

  const table = useTable({ defaultRowsPerPage: 50, defaultDense: true });

  const handleFilters = useCallback(
    (name: string, value: string) => {
      table.onResetPage();
      setFilters((prevState) => ({
        ...prevState,
        [name]: value,
      }));
    },
    [table]
  );

  const handleFilterStatus = useCallback(
    (event: React.SyntheticEvent, newValue: string) => {
      console.log('🚀 ~ file: view.tsx:127 ~ FourView ~ newValue:', newValue);
      handleFilters('type', newValue);
    },
    [handleFilters]
  );

  const dataFiltered = applyFilter({
    inputData: (tableData as RenderEvent[]) || [],
    comparator: getComparator(table.order, table.orderBy),
    filters,
    dateError: false,
  });

  const [targetDate, setTargetDate] = useState<number | undefined>(Date.now() + 30 * 1000);

  const [countdown] = useCountDown({
    targetDate,
    onEnd: async () => {
      await refetchTransactionEvent();
      setTargetDate(Date.now() + 30 * 1000);
    },
  });

  return (
    <Container maxWidth="xl" className="!mt-1">
      <Stack direction="row" alignItems="center" justifyContent="space-between">
        <Stack direction="row" alignItems="center" spacing={1}>
          <Typography variant="h4">
            <span className="underline mr-2">
              {TICK_MAP[paramTick].fromTokenSymbol} / {TICK_MAP[paramTick].toTokenSymbol}
            </span>{' '}
            Transaction History
          </Typography>
          <Chip size="small" variant="outlined" color="warning" label="Latest 50 records" />
        </Stack>
        <LoadingButton
          loading={isFetching}
          variant="outlined"
          startIcon={<Iconify icon="solar:refresh-bold" width={24} />}
          onClick={() => {
            refetchTransactionEvent();
            setTargetDate(Date.now() + 30 * 1000);
          }}
          suppressHydrationWarning
        >
          Refresh
        </LoadingButton>
      </Stack>

      {targetDate && (
        <Stack direction="row" alignItems="center" spacing={1} sx={{ mt: 2 }}>
          <PuffLoader speedMultiplier={0.875} color={warning.light} loading size={24} />
          <Typography
            sx={{
              fontSize: '0.875rem',
              color: grey[600],
            }}
          >
            {isFetching ? (
              'Refreshing...'
            ) : (
              <span>Refresh after {Math.round(countdown / 1000)} second(s)</span>
            )}
          </Typography>
        </Stack>
      )}

      <Card
        sx={{
          mt: 2,
          mb: 4,
        }}
      >
        <Tabs
          value={filters.type}
          onChange={handleFilterStatus}
          sx={{
            px: 2.5,
            boxShadow: (theme) => `inset 0 -2px 0 0 ${alpha(theme.palette.grey[500], 0.08)}`,
          }}
        >
          {STATUS_OPTIONS.map((tab) => (
            <Tab
              key={tab.value}
              iconPosition="end"
              value={tab.value}
              label={tab.label}
              icon={
                <Label
                  variant="outlined"
                  color={
                    ((tab.label === EVENT_ENUM.BuyEvent ||
                      tab.label === EVENT_ENUM.AcceptBidEvent) &&
                      'success') ||
                    ((tab.label === EVENT_ENUM.ListEvent ||
                      tab.label === EVENT_ENUM.CreateBidEvent) &&
                      'warning') ||
                    (tab.label === EVENT_ENUM.CancelListEvent && 'primary') ||
                    'default'
                  }
                >
                  {isLoading
                    ? '...'
                    : ['Buy', 'List', 'Create Bid', 'Accept Bid', 'Cancel List'].includes(tab.label)
                      ? tableData?.filter((user) => ORDER_TYPE_MAP[user.order_type] === tab.label)
                          .length
                      : tableData?.length}
                </Label>
              }
            />
          ))}
        </Tabs>

        <TableContainer sx={{ position: 'relative', overflow: 'unset' }}>
          <Scrollbar>
            <Table size={table.dense ? 'small' : 'medium'} sx={{ minWidth: 960 }}>
              <TableHeadCustom
                order={table.order}
                orderBy={table.orderBy}
                headLabel={TABLE_HEAD}
                rowCount={dataFiltered.length}
                numSelected={table.selected.length}
              />

              <TableBody>
                {dataFiltered
                  .slice(
                    table.page * table.rowsPerPage,
                    table.page * table.rowsPerPage + table.rowsPerPage
                  )
                  .map((row) => (
                    <OrderTableRow
                      key={`${row.order_id}-${row.order_type}-${row.tx_hash}`}
                      row={row}
                      tickInfo={TICK_MAP[paramTick]}
                      selected={table.selected.includes(row.order_id)}
                      onSelectRow={() => table.onSelectRow(row.order_id)}
                      onDeleteRow={() => {}}
                      onViewRow={() => {}}
                    />
                  ))}

                {isLoading && (
                  <TableRow>
                    {[...Array(6)].map((_, index) => (
                      <TableCell key={index}>
                        <Skeleton width="100%" height={24} variant="rounded" />
                      </TableCell>
                    ))}
                  </TableRow>
                )}

                {!isLoading && <TableNoData notFound={dataFiltered.length === 0} />}
              </TableBody>
            </Table>
          </Scrollbar>
        </TableContainer>

        {/* <TablePaginationCustom
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
          }}
          count={dataFiltered.length}
          page={table.page}
          rowsPerPage={table.rowsPerPage}
          onPageChange={table.onChangePage}
          onRowsPerPageChange={table.onChangeRowsPerPage}
          //
          dense
          onChangeDense={table.onChangeDense}
        /> */}
      </Card>
    </Container>
  );
}
