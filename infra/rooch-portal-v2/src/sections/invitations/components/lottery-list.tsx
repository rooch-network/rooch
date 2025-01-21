import dayjs from 'dayjs';
import { useState, useEffect, useCallback } from 'react';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useRoochClient, SessionKeyGuard, useCurrentSession } from '@roochnetwork/rooch-sdk-kit';

import {
  Box,
  Card,
  Table,
  Button,
  TableRow,
  TableBody,
  TableCell,
  Typography,
} from '@mui/material';

import { Scrollbar } from '../../../components/scrollbar';
import { getUTCOffset } from '../../../utils/format-time';
import { formatCoin } from '../../../utils/format-number';
import { useNetworkVariable } from '../../../hooks/use-networks';
import { GAS_COIN_DECIMALS } from '../../../config/constant';
import TableSkeleton from '../../../components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from '../../../components/table';

const options = [1, 5, 10, 0];

type ListType = {
  reward: number;
  timestamp: number;
};

export function InvitationLotteryList({
  table,
  ticket = 0,
  openCallback,
}: {
  table?: string;
  ticket: number;
  openCallback: () => void;
}) {
  const client = useRoochClient();
  const [data, setData] = useState<Array<ListType>>();
  const [loading, setLoading] = useState(false);
  const [opening, setOpening] = useState(false);
  const [ticketOption, setTicketOption] = useState(1);
  const session = useCurrentSession();
  // const [inviterCA, inviterModule, inviterObj] = useNetworkVariable('inviterCA');
  const inviterCfg = useNetworkVariable('inviter');

  const fetch = useCallback(() => {
    setLoading(true);
    client
      .listStates({
        accessPath: `/table/${table}`,
        stateOption: {
          decode: true,
        },
      })
      .then((result) => {
        setData(
          result.data.map((item) => {
            console.log(result);
            const view = (item.state.decoded_value!.value as any).value.value;
            return {
              address: view.address,
              reward: view.reward_amount,
              timestamp: view.timestamp,
            };
          })
        );
      })
      .catch((e) => {
        console.log(e);
      })
      .finally(() => setLoading(false));
  }, [client, table]);

  useEffect(() => {
    if (!table) {
      return;
    }

    fetch();
  }, [fetch, table]);

  const openTicket = async () => {
    setOpening(true);
    const tx = new Transaction();
    tx.callFunction({
      target: `${inviterCfg.address}::${inviterCfg.module}::lottery`,
      args: [
        Args.object(inviterCfg.obj(inviterCfg)),
        Args.u64(BigInt(ticketOption === 0 ? ticket : ticketOption)),
      ],
    });

    const result = await client.signAndExecuteTransaction({
      transaction: tx,
      signer: session!,
    });

    if (result.execution_info.status.type === 'executed') {
      openCallback();
      fetch();
    }
    setOpening(false);
    console.log(result);
  };

  return (
    <Card className="mt-4">
      <Box
        display="flex"
        justifyContent="space-between"
        alignItems="center"
        sx={{ p: 2, height: 60 }}
      >
        <Typography variant="h6">Activity History</Typography>
        {ticket === 0 || (
          <Box display="flex" alignItems="center">
            {options.map((item) => (
              <Button
                disabled={item !== 0 && ticket < item}
                variant={ticketOption === item ? 'contained' : 'outlined'}
                size="small"
                sx={{ mx: 0.5 }}
                onClick={() => {
                  setTicketOption(item);
                }}
              >
                {item === 0 ? 'All' : item}
              </Button>
            ))}
            <SessionKeyGuard onClick={openTicket}>
              <Button variant="outlined">Open Ticket</Button>
            </SessionKeyGuard>
          </Box>
        )}
      </Box>
      <Scrollbar sx={{ minHeight: 462 }}>
        <Table sx={{ minWidth: 720 }} size="medium">
          <TableHeadCustom
            headLabel={[
              {
                id: 'timestamp',
                label: (
                  <Box>
                    Timestamp <span className="text-xs ml-1">({getUTCOffset()})</span>
                  </Box>
                ),
              },
              { id: 'coin', label: 'RGAS' },
            ]}
          />
          <TableBody>
            {loading ? (
              <TableSkeleton col={2} row={10} rowHeight="69px" />
            ) : (
              <>
                {data?.map((item) => (
                  <TableRow key={item.timestamp}>
                    <TableCell>
                      {dayjs(Number(item.timestamp * 1000)).format('MMMM DD, YYYY HH:mm:ss')}
                    </TableCell>
                    {item.reward && (
                      <TableCell className="!text-xs">
                        {formatCoin(Number(item.reward), GAS_COIN_DECIMALS, 6)}
                      </TableCell>
                    )}
                  </TableRow>
                ))}
                <TableNoData
                  title="No Lottery History"
                  notFound={data?.length === 0 || data === undefined}
                />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
    </Card>
  );
}
