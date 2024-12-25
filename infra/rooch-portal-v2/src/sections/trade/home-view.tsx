'use client';

import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import { Table, TableBody, TableContainer } from '@mui/material';

import { Scrollbar } from 'src/components/scrollbar';
import { useSettingsContext } from 'src/components/settings';
import { TableHeadCustom } from 'src/components/table/table-head-custom';

export interface Tick {
  current_epoch: string;
  current_supply: string;
  epoch_count: string;
  epoch_records: {
    type: string;
    fields: {
      id: {
        id: string;
      };
      size: string;
    };
  };
  id: {
    id: string;
  };
  mint_fee: string;
  remain: string;
  start_time_ms: string;
  tick: string;
  total_supply: string;
  total_transactions: string;
  version: string;
}

// ----------------------------------------------------------------------
const TABLE_HEAD = [
  { id: 'name', label: 'Name' },
  { id: 'todayVolume	', label: 'Today Volume (SUI)', align: 'center' },
  { id: 'totalVolume', label: 'Total Volume (SUI)', align: 'right' },
  { id: 'totalSupply', label: 'Total Supply', align: 'right' },
  { id: 'action', label: 'Action', align: 'center' },
];

export default function MarketplaceHomeView() {
  const settings = useSettingsContext();
  // const { tickList: ticks, isFetching } = useMRCTicks();
  // const { tickTradeInfos, isLoadingTickTradeInfos } = useBatchMarketTradeData([
  //   'grow',
  //   'gold',
  // ]);

  return (
    <Container maxWidth="xl">
      <Typography variant="h4"> Marketplace </Typography>

      <TableContainer
        sx={{
          mt: 3,
          overflow: 'unset',
          '& .simplebar-content-wrapper': {
            borderRadius: '8px !important',
          },
        }}
      >
        <Scrollbar>
          <Table sx={{ minWidth: 800 }}>
            <TableHeadCustom
              headLabel={TABLE_HEAD}
              sx={{
                borderRadius: 8,
              }}
            />

            <TableBody>
              {/* {false ? (
                <TableRow>
                  <TableCell>
                    <Skeleton />
                  </TableCell>
                  <TableCell align="center">
                    <Skeleton />
                  </TableCell>
                  <TableCell align="right">
                    <Skeleton />
                  </TableCell>
                  <TableCell align="right">
                    <Skeleton />
                  </TableCell>
                  <TableCell align="right">
                    <Skeleton />
                  </TableCell>
                  <TableCell align="center">
                    <Skeleton />
                  </TableCell>
                </TableRow>
              ) : (
                tickTradeInfos
                  .sort((a, b) => Number(b.today_volume) - Number(a.today_volume))
                  .map((row) => {
                    const todayVolume = new BigNumber(row.today_volume);
                    const totalVolume = new BigNumber(row.total_volume);
                    const totalSupply =
                      ticks?.find((i) => i.tick.toLowerCase() === row.tick.toLowerCase())?.stat
                        .fields.current_supply || 0;
                    const isVerified = row.tick?.toLowerCase() === 'move';
                    return (
                      <TableRow key={row.tick}>
                        <TableCell>
                          <Typography
                            sx={{
                              fontWeight: 600,
                              fontSize: '1rem',
                              display: 'flex',
                              alignItems: 'center',
                            }}
                          >
                            {row.tick}
                            {isVerified && (
                              <Iconify
                                icon="solar:verified-check-bold"
                                color={secondary.main}
                                width={20}
                                sx={{
                                  ml: 1,
                                }}
                              />
                            )}
                          </Typography>
                        </TableCell>
                        <TableCell
                          align="center"
                          sx={{
                            fontWeight: 600,
                          }}
                        >
                          {fNumber(fromDust(todayVolume.toNumber(), SUI_DECIMALS).toNumber())}
                        </TableCell>
                        <TableCell
                          align="right"
                          sx={{
                            fontWeight: 600,
                          }}
                        >
                          {fNumber(fromDust(totalVolume.toNumber(), SUI_DECIMALS).toNumber())}
                        </TableCell>
                        <TableCell
                          align="right"
                          sx={{
                            fontWeight: 600,
                          }}
                        >
                          {totalSupply === 0 ? '--' : fNumber(totalSupply)}
                        </TableCell>
                        <TableCell align="center">
                          <Link href={`/marketplace/${row.tick?.toLowerCase()}`}>
                            <Button variant="outlined" color="success">
                              Trade
                            </Button>
                          </Link>
                        </TableCell>
                      </TableRow>
                    );
                  })
              )} */}
            </TableBody>
          </Table>
        </Scrollbar>
      </TableContainer>
    </Container>
  );
}
