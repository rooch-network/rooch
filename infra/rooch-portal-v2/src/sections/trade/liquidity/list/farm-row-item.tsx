import dayjs from 'dayjs';
import { Fragment, useState, useEffect, useCallback } from 'react';
import { Args, Transaction, type BalanceInfoView } from '@roochnetwork/rooch-sdk';
import {
  useRoochClient,
  SessionKeyGuard,
  useCurrentAddress,
  useSignAndExecuteTransaction,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import KeyboardArrowUpIcon from '@mui/icons-material/KeyboardArrowUp';
import KeyboardArrowDownIcon from '@mui/icons-material/KeyboardArrowDown';
import {
  Box,
  Card,
  Stack,
  Button,
  TableRow,
  Collapse,
  TableCell,
  Typography,
  CardContent,
  ListItemText,
} from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatByIntl, fromDust } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

import type { OwnerLiquidityItemType } from '../../hooks/use-owner-liquidity';

export type FarmRowItemType = {
  id: string;
  alive: boolean;
  endtime: number;
  assetTotalWeight: number;
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
  selectRow?: FarmRowItemType;
  onOpenStakeModal: (row: FarmRowItemType) => void;
  onOpenAddLiquidityModal: (row: FarmRowItemType) => void;
};

export default function FarmRowItem({
  row,
  onOpenStakeModal,
  onOpenAddLiquidityModal,
  selectRow,
}: RowItemProps) {
  const [openCollapse, setOpenCollapse] = useState(false);
  const client = useRoochClient();
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();
  const [staked, setStaked] = useState(0);
  const [harvest, setHarvest] = useState(0);
  const [rewardCoin, setRewardCoin] = useState<BalanceInfoView>();
  const { mutateAsync, isPending } = useSignAndExecuteTransaction();

  const { data: coinInfo } = useRoochClientQuery('getBalance', {
    owner: '0x3',
    coinType: row.reward,
  });

  const fetchHarvest = useCallback(() => {
    if (!openCollapse) {
      return;
    }
    client
      .executeViewFunction({
        target: `${dex.address}::liquidity_incentive::query_stake`,
        args: [Args.objectId(row.id), Args.address(currentAddress?.toStr() || '')],
        typeArgs: [row.x.type, row.y.type, row.reward],
      })
      .then((result) => {
        const s = result.return_values![0].decoded_value as number;
        setStaked(s);
        if (s > 0) {
          client
            .executeViewFunction({
              target: `${dex.address}::liquidity_incentive::query_harvest_token_amount`,
              args: [Args.address(currentAddress?.toStr() || ''), Args.objectId(row.id)],
              typeArgs: [row.x.type, row.y.type, row.reward],
            })
            .then((result) => {
              const s = result.return_values![0].decoded_value as number;
              setHarvest(s);
            });
        }
      });
  }, [client, dex, openCollapse, row, currentAddress]);

  useEffect(() => {
    fetchHarvest();
  }, [selectRow, fetchHarvest]);

  useEffect(() => {
    fetchHarvest();
  }, [fetchHarvest]);

  useEffect(() => {
    client
      .getBalance({
        owner: currentAddress?.toStr() || '',
        coinType: row.reward,
      })
      .then((result) => {
        setRewardCoin(result);
      });
  }, [openCollapse, client, currentAddress, row]);

  const handleHarvest = () => {
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::liquidity_incentive::harvest`,
      args: [Args.objectId(row.id)],
      typeArgs: [row.x.type, row.y.type, row.reward],
    });
    mutateAsync({
      transaction: tx,
    })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          fetchHarvest();
          toast.success('harvest success');
        } else {
          toast.error('harvest failed');
        }
      })
      .catch((e: any) => {
        console.log(e);
        toast.error('harvest failed');
      });
  };

  const handleAction = (target: 'harvest' | 'unstake') => {
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::liquidity_incentive::${target}`,
      args: [Args.objectId(row.id)],
      typeArgs: [row.x.type, row.y.type, row.reward],
    });
    mutateAsync({
      transaction: tx,
    })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          fetchHarvest();
          toast.success(`${target} success`);
        } else {
          toast.error(`${target} failed`);
        }
      })
      .catch((e: any) => {
        toast.error(`${target} failed`);
      });
  };

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
          <ListItemText
            primary={formatByIntl(
              fromDust(row.releasePerSecond, coinInfo?.decimals || 0).toString()
            )}
          />
        </TableCell>
        <TableCell>
          <ListItemText
            primary={formatByIntl(
              fromDust(row.assetTotalWeight, coinInfo?.decimals || 0).toString()
            )}
          />
        </TableCell>
        <TableCell>
          <ListItemText
            primary={dayjs(Number(row.endtime * 1000)).format('MMMM DD, YYYY HH:mm:ss')}
          />
        </TableCell>

        <TableCell align="right" sx={{ pr: 1 }}>
          {openCollapse ? <KeyboardArrowUpIcon /> : <KeyboardArrowDownIcon />}
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
                  <Button
                    variant="outlined"
                    size="small"
                    onClick={() => onOpenAddLiquidityModal(row)}
                  >
                    Get {row.x.name}-{row.y.name} LP
                  </Button>
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
                  <Stack direction="column">
                    <Typography className="text-gray-600 !text-sm !font-semibold">
                      Eligible ${rewardCoin?.symbol}:{' '}
                      {formatByIntl(fromDust(harvest, rewardCoin?.decimals || 0).toString())}
                    </Typography>
                    {staked > 0 && (
                      <SessionKeyGuard
                        onClick={() => {
                          handleAction(harvest > 0 ? 'harvest' : 'unstake');
                        }}
                      >
                        <LoadingButton
                          loading={isPending}
                          sx={{ mt: 1 }}
                          variant="outlined"
                          size="small"
                        >
                          {harvest > 0 && 'Harvest'}
                          {harvest < 1 && 'Unstake'}
                        </LoadingButton>
                      </SessionKeyGuard>
                    )}
                  </Stack>
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
                  <Stack direction="column">
                    <Typography className="text-gray-600 !text-sm !font-semibold">
                      Staked ${rewardCoin?.symbol}:{' '}
                      {formatByIntl(fromDust(staked, rewardCoin?.decimals || 0).toString())}
                    </Typography>
                    <Button
                      sx={{ mt: 1 }}
                      disabled={!row.liquidity || row.liquidity.fixedBalance === 0}
                      variant="outlined"
                      size="small"
                      onClick={() => onOpenStakeModal(row)}
                    >
                      Stake LP
                    </Button>
                  </Stack>
                </CardContent>
              </Card>
            </Stack>
          </Collapse>
        </TableCell>
      </TableRow>
    </Fragment>
  );
}
