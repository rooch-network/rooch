'use client';

import { useState, useEffect } from 'react';
import { Args } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import Typography from '@mui/material/Typography';
import { Tab, Box, Tabs, Card, Stack, CardHeader, CardContent } from '@mui/material';

import { useTabs } from '../../hooks/use-tabs';
import { fromDustToPrecision } from '../../utils/number';
import { AnimateCountUp } from '../../components/animate';
import { DashboardContent } from '../../layouts/dashboard';
import { useNetworkVariable } from '../../hooks/use-networks';
import { InvitationList } from './components/invitation-list';
import { InvitationLotteryList } from './components/lottery-list';

const TABS = [
  { label: 'Invitation List', value: 'invitation_records' },
  { label: 'Lottery Tickets', value: 'lottery_tickets' },
];

type inviterDataType = {
  invitationCount: number;
  invitationReward: number;
  lotteryTable: string;
  invitationTable: string;
  lotteryReward: number;
  remainingLotteryTicket: number;
};

export function InvitationsView() {
  // const [inviterCA, inviterModule, inviterObj] = useNetworkVariable('inviterCA');
  const inviter = useNetworkVariable('inviter');
  const currentAddress = useCurrentAddress();
  const tabs = useTabs('invitation_records');
  const [inviterData, setInviterData] = useState<inviterDataType>();

  const { data, refetch } = useRoochClientQuery('executeViewFunction', {
    target: `${inviter.address}::${inviter.module}::invitation_user_record`,
    args: [
      Args.object(inviter.obj(inviter)),
      Args.address(currentAddress?.genRoochAddress().toHexAddress() || ''),
    ],
  });

  useEffect(() => {
    if (!data || data.vm_status !== 'Executed') {
      return;
    }

    const dataView = (data.return_values![0].decoded_value as any).value;

    setInviterData({
      invitationCount: Number(dataView.total_invitations),
      invitationReward: Number(dataView.invitation_reward_amount),
      invitationTable: dataView.invitation_records.value.contents.value.handle.value.id as string,
      lotteryTable: dataView.lottery_records.value.contents.value.handle.value.id as string,
      remainingLotteryTicket: Number(dataView.remaining_luckey_ticket),
      lotteryReward: Number(dataView.lottery_reward_amount),
    });
  }, [data]);

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

      <Box
        display="flex"
        className="mt-4"
        justifyContent="space-between"
        alignItems="stretch"
        sx={{ gap: 2 }}
      >
        <Card sx={{ flex: 1 }}>
          <CardHeader title="🔗 Invites Sent" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <AnimateCountUp to={inviterData?.invitationCount || 0} sx={{ fontSize: '100px' }} />
          </CardContent>
        </Card>
        <Card sx={{ flex: 1 }}>
          <CardHeader title="💰 Your Earnings" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <AnimateCountUp
              to={Number(
                fromDustToPrecision(
                  inviterData ? inviterData.invitationReward + inviterData.lotteryReward : 0,
                  8
                )
              )}
              sx={{ fontSize: '100px' }}
            />
          </CardContent>
        </Card>
        <Card sx={{ flex: 1 }}>
          <CardHeader title="🎟️ Remaining Lottery Tickets" sx={{ mb: 1 }} />
          <CardContent className="!pt-0">
            <AnimateCountUp
              to={inviterData?.remainingLotteryTicket || 0}
              sx={{ fontSize: '100px' }}
            />
          </CardContent>
        </Card>
      </Box>

      {renderTabs}

      {tabs.value === 'lottery_tickets' && (
        <InvitationLotteryList
          table={inviterData?.lotteryTable}
          ticket={inviterData?.remainingLotteryTicket || 0}
          openCallback={() => refetch()}
        />
      )}

      {tabs.value === 'invitation_records' && (
        <InvitationList table={inviterData?.invitationTable} />
      )}
    </DashboardContent>
  );
}
