'use client';

import { useEffect } from 'react';
import {
  useCurrentAddress,
  useCurrentNetwork,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { Tab, Tabs, Stack, Card, CardHeader, CardContent, Box } from "@mui/material";
import Typography from '@mui/material/Typography';

import { Args } from '@roochnetwork/rooch-sdk';
import { useRouter } from '../../routes/hooks';
import { useTabs } from '../../hooks/use-tabs';
import { INVITER_ADDRESS_KEY } from '../../utils/inviter';
import { DashboardContent } from '../../layouts/dashboard';
import { useNetworkVariable } from '../../hooks/use-networks';
import { AnimateCountUp } from "../../components/animate";
import { InviterLotteryList } from "./components/lottery-list";
import { InvitationList } from "./components/invitation-list";

const TABS = [
  { label: 'Lottery Tickets', value: 'lottery_tickets' },
  { label: 'Invitation List', value: 'invitation_records' },
];

export function InviterView({ inviterAddress }: { inviterAddress?: string }) {
  const router = useRouter();
  const [inviterCA, inviterModule, inviterObj] = useNetworkVariable('inviterCA');
  const currentAddress = useCurrentAddress();
  const tabs = useTabs('lottery_tickets');

  useEffect(() => {
    if (inviterAddress && inviterAddress !== currentAddress?.toStr()) {
      window.localStorage.setItem(INVITER_ADDRESS_KEY, inviterAddress);
      router.push(`/setting`);
    }
  }, [currentAddress, inviterAddress, router]);

  const { data } = useRoochClientQuery('executeViewFunction', {
    target: `${inviterCA}::${inviterModule}::invitation_user_record`,
    args: [
      Args.object({
        address: inviterCA,
        module: inviterModule,
        name: inviterObj,
      }),
    ],
  });

  console.log(data);

  const renderTabs = (
    <Tabs value={tabs.value} onChange={tabs.onChange} sx={{ mb: { xs: 1, md: 1 } }}>
      {TABS.map((tab) => (
        <Tab key={tab.value} value={tab.value} label={tab.label} />
      ))}
    </Tabs>
  );

  return (
    <DashboardContent maxWidth="xl">
      <Stack flexDirection="row" justifyContent="space-between">
        <Typography variant="h4">Invitation Overview</Typography>
      </Stack>

      <Box display="flex" className="mt-4" justifyContent="space-between" alignItems="stretch" sx={{ gap: 2 }}>
        <Card sx={{ flex: 1 }}>
          <CardHeader title="🔗 Invites Sent" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <AnimateCountUp to={100} sx={{ fontSize: '100px' }} />
          </CardContent>
        </Card>
        <Card sx={{ flex: 1 }}>
          <CardHeader title="💰 Your Earnings" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <AnimateCountUp to={123566} sx={{ fontSize: '100px' }} />
          </CardContent>
        </Card>
        <Card sx={{ flex: 1 }}>
          <CardHeader title="🎟️ Remaining Lottery Tickets" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <AnimateCountUp to={100} sx={{ fontSize: '100px' }} />
          </CardContent>
        </Card>
      </Box>

      {renderTabs}

      {tabs.value === 'lottery_tickets' && <InviterLotteryList/>}

      {tabs.value === 'invitation_records' && <InvitationList/>}
    </DashboardContent>
  );
}
