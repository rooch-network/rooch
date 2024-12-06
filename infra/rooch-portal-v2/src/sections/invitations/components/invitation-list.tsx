import dayjs from 'dayjs';
import { useState, useEffect } from "react";
import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";

import {
  Box,
  Card,
  Table,
  Tooltip,
  TableRow,
  TableBody,
  TableCell,
  Typography,
} from '@mui/material';

import { shortAddress } from '../../../utils/address';
import { Scrollbar } from '../../../components/scrollbar';
import { getUTCOffset } from '../../../utils/format-time';
import { formatCoin } from '../../../utils/format-number';
import { ROOCH_GAS_COIN_DECIMALS } from '../../../config/constant';
import TableSkeleton from '../../../components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from '../../../components/table';

type ListType = {
  address: string
  reward: number
  timestamp: number
}

export function InvitationList({ table }: { table?: string }) {
  const client = useRoochClient()
  const [loading, setLoading] = useState(false)
  const [data, setData] = useState<Array<ListType>>()

  useEffect(() => {
    if (!table) {
      return
    }

    setLoading(true)
    client.listStates({
      accessPath: `/table/${table}`,
      stateOption: {
        decode: true
      }
    }).then((result) => {
      setData(result.data.map((item) => {
        const view = ((item.state.decoded_value!.value) as any).value.value
        return {
          address: view.address,
          reward: view.reward_amount,
          timestamp: view.timestamp,
        }
      }))
    }).catch((e) => {
      console.log(e)
    }).finally(() => setLoading(false))
  }, [table, client]);

  return (
    <Card className="mt-4">
      <Box sx={{height: 60, p:2}} >
        <Typography variant="h6" >Activity History</Typography>
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
                        {formatCoin(Number(item.reward), ROOCH_GAS_COIN_DECIMALS, 6)}
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
