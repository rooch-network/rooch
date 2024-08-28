'use client';

import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Chip, Stack, CardHeader, CardContent } from '@mui/material';

import { RouterLink } from 'src/routes/components';

import { BitcoinAddressToRoochAddress } from 'src/utils/address';

import { DashboardContent } from 'src/layouts/dashboard';

import AssetsTableCard from '../assets/components/assets-table-card';
import TransactionsTableCard from '../transactions/components/transactions-table-card';

export function AccountView({ address }: { address: string }) {
  const { data: transactionsList, isPending: isTransactionsPending } = useRoochClientQuery(
    'queryTransactions',
    {
      filter: {
        sender: BitcoinAddressToRoochAddress(address).toHexAddress(),
      },
      limit: '5',
    }
  );

  return (
    <DashboardContent maxWidth="xl">
      <Card>
        <CardHeader title="Account Info" sx={{ mb: 1 }} />
        <CardContent className="!pt-0">
          <Stack spacing={2}>
            <Stack direction="row" alignItems="center">
              {/* <Box>Bitcoin Address</Box> */}
              <Chip
                className="w-fit !cursor-pointer"
                label={address}
                variant="soft"
                color="secondary"
                component={RouterLink}
                href={`/account/${address}`}
              />
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip
                className="w-fit"
                label={BitcoinAddressToRoochAddress(address).toStr()}
                variant="soft"
                color="default"
              />
              <Box className="text-gray-400 text-sm font-medium">(Rooch Address)</Box>
            </Stack>
          </Stack>
        </CardContent>
      </Card>

      <AssetsTableCard dense address={address} />
      <TransactionsTableCard
        dense
        address={address}
        isPending={isTransactionsPending}
        transactionsList={transactionsList}
      />
    </DashboardContent>
  );
}
