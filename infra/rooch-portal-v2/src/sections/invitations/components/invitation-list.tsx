import dayjs from 'dayjs';
import { useMemo, useState, useEffect } from 'react';
import { useRoochClient, useCurrentAddress, useCurrentNetwork } from '@roochnetwork/rooch-sdk-kit';

import {
  Box,
  Card,
  Table,
  Button,
  Tooltip,
  TableRow,
  TableBody,
  TableCell,
  Typography,
} from '@mui/material';

import { toast } from 'src/components/snackbar';

import { shortAddress } from '../../../utils/address';
import { Scrollbar } from '../../../components/scrollbar';
import { getUTCOffset } from '../../../utils/format-time';
import { formatCoin } from '../../../utils/format-number';
import { GAS_COIN_DECIMALS } from '../../../config/constant';
import TableSkeleton from '../../../components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from '../../../components/table';
import { getShareLink, getTwitterShareText } from '../../../utils/inviter';

type ListType = {
  address: string;
  reward: number;
  timestamp: number;
};

export function InvitationList({ table }: { table?: string }) {
  const network = useCurrentNetwork();
  const address = useCurrentAddress();
  const client = useRoochClient();
  const [loading, setLoading] = useState(false);
  const [data, setData] = useState<Array<ListType>>();
  const XText = useMemo(() => getTwitterShareText(network, address), [network, address]);
  const shareText = useMemo(() => getShareLink(network, address), [network, address]);

  useEffect(() => {
    if (!table) {
      return;
    }

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
  }, [table, client]);

  return (
    <Card className="mt-4">
      <Box
        display="flex"
        justifyContent="space-between"
        alignItems="center"
        sx={{ p: 2, height: 60 }}
      >
        <Typography variant="h6">Activity History</Typography>
        {address && (
          <Box display="flex" alignItems="center">
            <Button
              size="small"
              variant="soft"
              color="error"
              sx={{ mx: 0.5 }}
              onClick={async () => {
                await navigator.clipboard.writeText(shareText);
                toast.success('copy success');
              }}
            >
              Copy Invite Link
            </Button>

            <Button
              size="small"
              variant="soft"
              color="error"
              sx={{ mx: 0.5 }}
              onClick={() => {
                window.open(
                  `https://twitter.com/intent/tweet?text=${encodeURIComponent(XText)}`,
                  '_blank'
                );
              }}
            >
              Invite With Twitter
            </Button>
          </Box>
        )}
      </Box>
      <Scrollbar sx={{ minHeight: 462 }}>
        <Table sx={{ minWidth: 720 }} size="medium">
          <TableHeadCustom
            headLabel={[
              { id: 'address', label: 'Address' },
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
              <TableSkeleton col={3} row={10} rowHeight="69px" />
            ) : (
              <>
                {data?.map((item) => (
                  <TableRow key={item.address}>
                    <TableCell width="256px">
                      <Typography className="!font-mono !font-medium">
                        <Tooltip title={item.address} arrow>
                          <span>{shortAddress(item.address, 8, 6)}</span>
                        </Tooltip>
                      </Typography>
                    </TableCell>
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
                <TableNoData title="No Invitions Found" notFound={data === undefined} />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
    </Card>
  );
}
