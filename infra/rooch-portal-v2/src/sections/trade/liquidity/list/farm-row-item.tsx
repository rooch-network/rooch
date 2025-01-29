import { Args, type BalanceInfoView } from '@roochnetwork/rooch-sdk';

import {
  useCurrentAddress,
  useRoochClient,
  useRoochClientQuery,
  WalletGuard,
} from '@roochnetwork/rooch-sdk-kit';

import {
  Box,
  Button,
  TableRow,
  TableCell,
  ListItemText,
  Collapse,
  Stack,
  Card,
  CardHeader,
  CardContent,
  Typography,
  Icon,
} from '@mui/material';
import dayjs from 'dayjs';
import { Fragment, useEffect, useMemo, useState } from 'react';
import { LoadingButton } from '@mui/lab';
import KeyboardArrowDownIcon from '@mui/icons-material/KeyboardArrowDown';
import KeyboardArrowUpIcon from '@mui/icons-material/KeyboardArrowUp';
import type { OwnerLiquidityItemType } from '../../hooks/use-owner-liquidity';
import { useNetworkVariable } from 'src/hooks/use-networks';

export type FarmRowItemType = {
  id: string;
  alive: boolean;
  endtime: number;
  assetTotalWeight: number;
  harvestIndex: number;
  releasePerSecond: number;
  x: {
    type: string;
    name: string;
  };
  y: { type: string; name: string };
  reward: string;
  liquidity?: OwnerLiquidityItemType;
};

type RowItemProps = {
  row: FarmRowItemType;
  onOpenStakeModal: (row: FarmRowItemType) => void;
};

export default function FarmRowItem({ row, onOpenStakeModal }: RowItemProps) {
  const [openCollapse, setOpenCollapse] = useState(false);
  const client = useRoochClient();
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();
  const [harvest, setHarvest] = useState(0);

  useEffect(() => {
    if (!openCollapse || harvest) {
      return;
    }
    client
      .executeViewFunction({
        target: `${dex.address}::liquidity_incentive::query_harvest_token_amount`,
        args: [Args.address(currentAddress?.toStr() || ''), Args.objectId(row.id)],
        typeArgs: [row.x.type, row.y.type, row.reward],
      })
      .then((result) => {
        const s = result.return_values![0].decoded_value as number;
        setHarvest(s);
        console.log(result);
      });
  }, [openCollapse]);

  return (
    <Fragment key={row.id}>
      <TableRow
        sx={{ cursor: 'pointer' }}
        onClick={() => {
          setOpenCollapse(!openCollapse);
        }}
      >
        <TableCell width="300px">
          <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
            <ListItemText primary={`${row.x.name} - ${row.y.name}`} />
          </Box>
        </TableCell>
        <TableCell>
          <ListItemText primary={row.harvestIndex} />
        </TableCell>
        <TableCell>
          <ListItemText primary={row.releasePerSecond} />
        </TableCell>
        <TableCell>
          <ListItemText primary={row.assetTotalWeight} />
        </TableCell>
        <TableCell>
          <ListItemText
            primary={dayjs(Number(row.endtime * 1000)).format('MMMM DD, YYYY HH:mm:ss')}
          />
        </TableCell>

        <TableCell align="right" sx={{ pr: 1 }}>
          {openCollapse ? <KeyboardArrowUpIcon /> : <KeyboardArrowDownIcon />}
          {/* <WalletGuard
            onClick={() => {
              onOpenViewModal(row);
            }}
          >
            <Button variant="outlined" size="small">
              Remove
            </Button>
          </WalletGuard> */}
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell sx={{ p: 0.5 }} style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={6}>
          <Collapse in={openCollapse} timeout="auto" unmountOnExit>
            <Stack
              sx={{ p: 0.5, m: 0.5 }}
              className="bg-gray-50 rounded"
              direction="row"
              spacing={1}
            >
              <Card sx={{ width: '100%' }}>
                <CardContent
                  sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'center',
                    height: '100%',
                  }}
                >
                  <LoadingButton variant="outlined" size="small">
                    Get {row.x.name}-{row.y.name} LP
                  </LoadingButton>
                </CardContent>
              </Card>
              <Card sx={{ width: '100%' }}>
                <CardContent>
                  <Typography className="text-gray-600 !text-sm !font-semibold">
                    Balance:
                  </Typography>
                  <LoadingButton variant="outlined" size="small">
                    Harvest
                  </LoadingButton>
                </CardContent>
              </Card>
              <Card sx={{ width: '100%' }}>
                <CardContent
                  sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'center',
                    height: '100%',
                  }}
                >
                  <Button
                    disabled={!row.liquidity || row.liquidity.fixedBalance === 0}
                    variant="outlined"
                    size="small"
                    onClick={() => onOpenStakeModal(row)}
                  >
                    Stake LP
                  </Button>
                </CardContent>
              </Card>
            </Stack>
          </Collapse>
        </TableCell>
      </TableRow>
    </Fragment>
  );
}
