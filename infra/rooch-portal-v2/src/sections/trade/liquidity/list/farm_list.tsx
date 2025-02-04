import { useMemo, useState } from 'react';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Card, Table, TableBody } from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { Scrollbar } from 'src/components/scrollbar';
import WalletGuard from 'src/components/guard/WalletGuard';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import FarmRowItem from './farm-row-item';
import AddSrakeModal from './add-stake-modal';
import AddLiquidityModal from './add-liquidity-modal';
import { useAllLiquidity } from '../../hooks/use-all-liquidity';
import { useOwnerLiquidity } from '../../hooks/use-owner-liquidity';

import type { FarmRowItemType } from './farm-row-item';

const headerLabel = [
  { id: 'lp', label: 'LP' },
  { id: 'harvest_index', label: 'Harvest Index' },
  { id: 'release_per_second', label: 'Release Per Second' },
  { id: 'asset_total_weight', label: 'Asset Total Weight' },
  { id: 'endtime', label: 'Endtime' },
  { id: 'action', label: '', align: 'right' },
];

export default function FarmList() {
  const currentAddress = useCurrentAddress();
  const dex = useNetworkVariable('dex');
  const [openStakeModal, setOpenStakeModal] = useState(false);
  const [openAddLiquidityModal, setOpenAddLiquidityModal] = useState(false);
  const [selectedRow, setSelectedRow] = useState<FarmRowItemType>();
  const { lpTokens } = useOwnerLiquidity();
  const { lpTokens: allLPTokens } = useAllLiquidity(200);

  const { data: farms } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type: `${dex.address}::liquidity_incentive::FarmingAsset`,
    },
  });

  const resolvedFarms = useMemo(() => {
    if (!farms) {
      return [];
    }
    return farms.data.map((item) => {
      const view = item.decoded_value!.value;
      const types = item.object_type
        .replace(`${dex.address}::liquidity_incentive::FarmingAsset<`, '')
        .trim()
        .split(',');
      const x = {
        type: types[0].trim(),
        name: types[0].split('::')[2].trim(),
      };
      const y = {
        type: types[1].trim(),
        name: types[1].split('::')[2].trim(),
      };
      return {
        id: item.id,
        alive: view.alive as boolean,
        endtime: view.end_time as number,
        assetTotalWeight: view.asset_total_weight as number,
        harvestIndex: view.harvest_index as number,
        releasePerSecond: view.release_per_second as number,
        x,
        y,
        reward: types[2].replaceAll('>', '').trim(),
        liquidity: lpTokens.find((item) => item.x.type === x.type && item.y.type === y.type),
      };
    });
  }, [farms, lpTokens, dex.address]);

  const {
    data: assetsList,
    isPending,
    refetch: refetchAssetsList,
  } = useRoochClientQuery(
    'getBalances',
    {
      owner: currentAddress?.toStr() || '',
    },
    { refetchInterval: 5000 }
  );

  const handleOpenStakeModal = (row: FarmRowItemType) => {
    setSelectedRow(row);
    setOpenStakeModal(true);
  };

  const handleCloseStakeModal = () => {
    setOpenStakeModal(false);
    setSelectedRow(undefined);
  };

  const handleOpenAddLiquidityModal = (row: FarmRowItemType) => {
    setSelectedRow(row);
    setOpenAddLiquidityModal(true);
  };

  const handleCloseAddLiquidityModal = () => {
    setOpenAddLiquidityModal(false);
    setSelectedRow(undefined);
  };

  return (
    <WalletGuard>
      <Card className="mt-4">
        <Scrollbar sx={{ minHeight: 462 }}>
          <Table sx={{ minWidth: 720 }} size="medium">
            <TableHeadCustom headLabel={headerLabel} />

            <TableBody>
              {isPending ? (
                <TableSkeleton col={5} row={5} rowHeight="77px" />
              ) : (
                <>
                  {resolvedFarms?.map((row) => (
                    <FarmRowItem
                      key={row.id}
                      row={row}
                      onOpenAddLiquidityModal={handleOpenAddLiquidityModal}
                      onOpenStakeModal={handleOpenStakeModal}
                      selectRow={selectedRow}
                    />
                  ))}
                  <TableNoData title="No Coins Found" notFound={resolvedFarms?.length === 0} />
                </>
              )}
            </TableBody>
          </Table>
        </Scrollbar>

        {selectedRow && (
          <AddSrakeModal
            open={openStakeModal}
            onClose={handleCloseStakeModal}
            row={selectedRow}
            key={openStakeModal ? 'open' : 'closed'}
          />
        )}
        {selectedRow && (
          <AddLiquidityModal
            open={openAddLiquidityModal}
            onClose={handleCloseAddLiquidityModal}
            row={
              allLPTokens.find(
                (item) => item.x.type === selectedRow.x.type && item.y.type === selectedRow.y.type
              )!
            }
            key={openAddLiquidityModal ? 'open' : 'closed'}
          />
        )}
      </Card>
    </WalletGuard>
  );
}
