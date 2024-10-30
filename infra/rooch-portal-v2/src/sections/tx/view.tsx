'use client';

import type { ReactNode } from 'react';

import dayjs from 'dayjs';
import { useMemo } from 'react';
import relativeTime from 'dayjs/plugin/relativeTime';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { duotoneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';

import {
  Tab,
  Box,
  Card,
  Tabs,
  Chip,
  Stack,
  Button,
  Divider,
  Skeleton,
  CardHeader,
  CardContent,
} from '@mui/material';

import { useRouter } from 'src/routes/hooks';
import { RouterLink } from 'src/routes/components';

import { useTabs } from 'src/hooks/use-tabs';

import { formatCoin } from 'src/utils/format-number';

import { varAlpha } from 'src/theme/styles';
import { DashboardContent } from 'src/layouts/dashboard';
import { ROOCH_GAS_COIN_DECIMALS } from 'src/config/constant';

import { Iconify } from 'src/components/iconify';

import {
  TRANSACTION_TYPE_MAP,
  TRANSACTION_ACTION_TYPE_MAP,
  TRANSACTION_STATUS_TYPE_MAP,
} from '../transactions/constant';

dayjs.extend(relativeTime);

const TX_VIEW_TABS = [
  { label: 'Overview', value: 'overview' },
  { label: 'Raw JSON', value: 'raw' },
];

function PropsKeyItem({ itemKey }: { itemKey: string }) {
  return <Box className="text-sm font-semibold text-gray-600 w-48">{itemKey}</Box>;
}

function PropsValueItem({ children, loading }: { children: ReactNode; loading?: boolean }) {
  if (loading) {
    return <Skeleton width="160px" height="16px" />;
  }
  return children;
}

export function TxView({ hash }: { hash: string }) {
  const tabs = useTabs('overview');
  const router = useRouter();

  const { data: transactionDetail, isPending } = useRoochClientQuery('queryTransactions', {
    filter: {
      tx_hashes: [hash],
    },
  });

  const txDetail = useMemo(() => transactionDetail?.data[0], [transactionDetail]);

  const renderTabs = (
    <Tabs value={tabs.value} onChange={tabs.onChange} sx={{ mb: { xs: 2, md: 2 } }}>
      {TX_VIEW_TABS.map((tab) => (
        <Tab key={tab.value} value={tab.value} label={tab.label} />
      ))}
    </Tabs>
  );

  return (
    <DashboardContent maxWidth="xl">
      <Button
        className="w-fit"
        onClick={() => {
          router.back();
        }}
        startIcon={<Iconify icon="eva:arrow-ios-back-fill" width={16} />}
      >
        Back
      </Button>
      <Card className="mt-4">
        <CardHeader title="Transactions" subheader={hash} sx={{ mb: 3 }} />

        <Divider />

        <CardContent className="!pt-0">
          {renderTabs}
          {tabs.value === 'overview' && (
            <Stack
              spacing={2}
              className="p-4"
              sx={{
                borderRadius: 2,
                bgcolor: (theme) => varAlpha(theme.vars.palette.grey['500Channel'], 0.04),
                border: (theme) => `dashed 1px ${theme.vars.palette.divider}`,
              }}
            >
              <Stack direction="row" alignItems="center">
                <PropsKeyItem itemKey="Order" />
                <PropsValueItem loading={isPending}>
                  {txDetail && (
                    <Box>
                      <Chip
                        label={txDetail.transaction.sequence_info.tx_order}
                        size="small"
                        variant="outlined"
                        color="default"
                      />
                    </Box>
                  )}
                </PropsValueItem>
              </Stack>

              <Stack direction="row" alignItems="center">
                <PropsKeyItem itemKey="Type" />
                <PropsValueItem loading={isPending}>
                  {txDetail && (
                    <Box>
                      <Chip
                        label={TRANSACTION_TYPE_MAP[txDetail.transaction.data.type].text}
                        size="small"
                        variant="soft"
                        color={TRANSACTION_TYPE_MAP[txDetail.transaction.data.type].color}
                      />
                    </Box>
                  )}
                </PropsValueItem>
              </Stack>

              {txDetail?.execution_info && (
                <Stack direction="row" alignItems="center">
                  <PropsKeyItem itemKey="Status" />
                  <PropsValueItem loading={isPending}>
                    <Box>
                      {txDetail && (
                        <Chip
                          label={
                            TRANSACTION_STATUS_TYPE_MAP[txDetail.execution_info.status.type].text
                          }
                          size="small"
                          variant="soft"
                          color={
                            TRANSACTION_STATUS_TYPE_MAP[txDetail.execution_info.status.type].color
                          }
                        />
                      )}
                    </Box>
                  </PropsValueItem>
                </Stack>
              )}

              <Stack direction="row" alignItems="center">
                <PropsKeyItem itemKey="Timestamp" />
                <PropsValueItem loading={isPending}>
                  {txDetail && (
                    <Box className="text-sm font-semibold">
                      {dayjs(Number(txDetail.transaction.sequence_info.tx_timestamp)).fromNow()}

                      <span className="text-gray-500 ml-2">
                        (
                        {dayjs(Number(txDetail.transaction.sequence_info.tx_timestamp)).format(
                          'MMMM DD, YYYY HH:mm:ss   UTC Z'
                        )}
                        )
                      </span>
                    </Box>
                  )}
                </PropsValueItem>
              </Stack>

              {txDetail && txDetail.transaction.data.type === 'l2_tx' && (
                <Stack direction="row" alignItems="center">
                  <PropsKeyItem itemKey="Action Type" />
                  <PropsValueItem loading={isPending}>
                    <Box>
                      <Chip
                        label={
                          TRANSACTION_ACTION_TYPE_MAP[txDetail.transaction.data.action_type].text
                        }
                        size="small"
                        variant="outlined"
                        color={
                          TRANSACTION_ACTION_TYPE_MAP[txDetail.transaction.data.action_type].color
                        }
                      />
                    </Box>
                  </PropsValueItem>
                </Stack>
              )}

              {txDetail && txDetail.transaction.data.type === 'l2_tx' && (
                <Stack direction="row" alignItems="flex-start">
                  <PropsKeyItem itemKey="Sender" />
                  <PropsValueItem loading={isPending}>
                    <Stack spacing={1}>
                      {txDetail && (
                        <>
                          <Chip
                            className="w-fit !cursor-pointer"
                            label={txDetail.transaction.data.sender_bitcoin_address}
                            size="small"
                            variant="soft"
                            color="secondary"
                            component={RouterLink}
                            href={`/account/${txDetail.transaction.data.sender_bitcoin_address}`}
                          />
                          <Stack direction="row" alignItems="center" spacing={0.5}>
                            <Chip
                              className="w-fit"
                              label={txDetail?.transaction.data.sender}
                              size="small"
                              variant="soft"
                              color="default"
                            />
                            <Box className="text-gray-400 text-sm font-medium">(Rooch Address)</Box>
                          </Stack>
                        </>
                      )}
                    </Stack>
                  </PropsValueItem>
                </Stack>
              )}

              {txDetail && txDetail.transaction.data.type === 'l2_tx' && (
                <Stack direction="row" alignItems="center">
                  <PropsKeyItem itemKey="Sequence Number" />
                  <PropsValueItem loading={isPending}>
                    {txDetail && (
                      <Box>
                        <Chip
                          label={txDetail.transaction.data.sequence_number}
                          size="small"
                          variant="outlined"
                          color="default"
                        />
                      </Box>
                    )}
                  </PropsValueItem>
                </Stack>
              )}

              {txDetail?.execution_info && (
                <Stack direction="row" alignItems="center">
                  <PropsKeyItem itemKey="Gas Used" />
                  <PropsValueItem loading={isPending}>
                    {txDetail && (
                      <Box className="text-sm font-semibold">
                        {formatCoin(
                          Number(txDetail.execution_info.gas_used),
                          ROOCH_GAS_COIN_DECIMALS,
                          8
                        )}{' '}
                        RGAS
                      </Box>
                    )}
                  </PropsValueItem>
                </Stack>
              )}

              {/* <Stack direction="row" alignItems="center">
              <PropsKeyItem itemKey="Action Call args" />
              <Box>5959155</Box>
            </Stack> */}
            </Stack>
          )}
          {tabs.value === 'raw' && (
            <Stack>
              <SyntaxHighlighter
                language="json"
                style={duotoneLight}
                customStyle={{
                  whiteSpace: 'pre-wrap',
                  width: '100%',
                  borderRadius: '8px',
                  wordBreak: 'break-all',
                  overflowWrap: 'break-word',
                }}
                wrapLines
                wrapLongLines
              >
                {JSON.stringify(txDetail, null, 2)}
              </SyntaxHighlighter>
            </Stack>
          )}
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
