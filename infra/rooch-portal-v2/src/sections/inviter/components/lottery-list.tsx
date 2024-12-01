import {
  Box,
  Button,
  Card,
  CardHeader,
  Table,
  TableBody,
  TableCell,
  TableRow,
  Tooltip,
  Typography
} from "@mui/material";
import dayjs from "dayjs";
import { LoadingButton } from "@mui/lab";
import { Scrollbar } from "../../../components/scrollbar";
import { TableHeadCustom, TableNoData } from "../../../components/table";
import { getUTCOffset } from "../../../utils/format-time";
import TableSkeleton from "../../../components/skeleton/table-skeleton";
import { shortAddress } from "../../../utils/address";
import { formatCoin } from "../../../utils/format-number";
import { ROOCH_GAS_COIN_DECIMALS } from "../../../config/constant";

const data = [{
  address: 'tb1q04uaa0mveqtt4y0sltuxtauhlyl8ctstfjazv0',
  rgas: 111,
  timestamp: 1733064513
}]


export function InviterLotteryList() {
  console.log('hahahh')
  return (
    <Card className="mt-4">
        <Box display="flex" justifyContent="space-between" alignItems="center" sx={{ p: 2 }}>
          <Typography variant="h6">Activity History</Typography>
          <Box display="flex" alignItems="center">
            <Button variant="outlined" size="small" sx={{ mx: 0.5 }}>1</Button>
            <Button variant="outlined" size="small" sx={{ mx: 0.5 }}>5</Button>
            <Button variant="outlined" size="small" sx={{ mx: 0.5 }}>10</Button>
            <Button variant="outlined" size="small" sx={{ mx: 0.5 }}>All</Button>
            <LoadingButton variant="contained" sx={{ ml: 1 }}>Open Ticket</LoadingButton>
          </Box>
      </Box>
      <Scrollbar sx={{ minHeight: 462 }}>
        <Table sx={{ minWidth: 720 }} size='medium'>
          <TableHeadCustom
            headLabel={[
              { id: 'address', label: 'Address' },
              { id: 'coin', label: 'RGAS' },
              {
                id: 'timestamp',
                label: (
                  <Box>
                    Timestamp <span className="text-xs ml-1">({getUTCOffset()})</span>
                  </Box>
                ),
              },
            ]}
          />
          <TableBody>
            {false ? (
              <TableSkeleton col={6} row={10} rowHeight="69px" />
            ) : (
              <>
                {data.map((item) => (
                  <TableRow key={item.address}>
                    <TableCell width="256px">
                      <Typography className="!font-mono !font-medium">
                        <Tooltip title={item.address} arrow>
                          <span>{shortAddress(item.address, 8, 6)}</span>
                        </Tooltip>
                      </Typography>
                    </TableCell>
                    <TableCell>
                      {dayjs(Number(item.timestamp)).format(
                        'MMMM DD, YYYY HH:mm:ss'
                      )}
                    </TableCell>
                    {item.rgas && (
                      <TableCell className="!text-xs">
                        {formatCoin(
                          Number(item.rgas),
                          ROOCH_GAS_COIN_DECIMALS,
                          6
                        )}
                      </TableCell>
                    )}
                  </TableRow>
                ))}
                <TableNoData
                  title="No Transaction Found"
                  notFound={data.length === 0}
                />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
    </Card>
  )
}