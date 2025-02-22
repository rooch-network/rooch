import { useState } from 'react';

import { Card, Table, TableBody } from '@mui/material';

import { Scrollbar } from 'src/components/scrollbar';
import WalletGuard from 'src/components/guard/WalletGuard';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import type { OwnerLiquidityItemType } from 'src/sections/trade/hooks/use-owner-liquidity';
import { useAllLiquidity } from 'src/sections/trade/hooks/use-all-liquidity';
import { useOwnerLiquidity } from 'src/sections/trade/hooks/use-owner-liquidity';
import AddLiquidityModal from './add-liquidity-modal';
import RemoveLiquidityModal from './remove-liquidity-modal';
import OwnerLiquidityRowItem from './owner-liquidity-row-item';

const headerLabel = [
  { id: 'lp', label: 'LP' },
  { id: 'x', label: 'X' },
  { id: 'y', label: 'Y' },
  { id: 'balance', label: 'Balance' },
  { id: 'supply', label: 'Supply' },
  { id: 'action', label: 'Action', align: 'right' },
];

export default function OwnerLiquidityList() {
  const { lpTokens, isPending } = useOwnerLiquidity();

  const [openRemoveLiquidityModal, setOpenRemoveLiquidityModal] = useState(false);
  const [openAddLiquidityModal, setOpenAddLiquidityModal] = useState(false);
  const [selectedRow, setSelectedRow] = useState<OwnerLiquidityItemType>();
  const { lpTokens: allLPTokens } = useAllLiquidity(200);

  const handleRemoveModal = (row: OwnerLiquidityItemType) => {
    setSelectedRow(row);
    setOpenRemoveLiquidityModal(true);
  };

  const handleCloseRemoveModal = () => {
    setOpenRemoveLiquidityModal(false);
    setSelectedRow(undefined);
  };

  const handleAddModal = (row: OwnerLiquidityItemType) => {
    setSelectedRow(row);
    setOpenAddLiquidityModal(true);
  };

  const handleCloseAddModal = () => {
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
                  {lpTokens?.map((row) => (
                    <OwnerLiquidityRowItem
                      key={row.coin_type}
                      row={row}
                      onOpenAddModal={handleAddModal}
                      onOpenRemoveModal={handleRemoveModal}
                    />
                  ))}
                  <TableNoData title="No Liquidity Found" notFound={lpTokens?.length === 0} />
                </>
              )}
            </TableBody>
          </Table>
        </Scrollbar>
        {selectedRow && (
          <AddLiquidityModal
            open={openAddLiquidityModal}
            onClose={handleCloseAddModal}
            row={
              allLPTokens.find(
                (item) => item.x.type === selectedRow.x.type && item.y.type === selectedRow.y.type
              )!
            }
            key={openAddLiquidityModal ? 'open' : 'closed'}
          />
        )}
        {selectedRow && (
          <RemoveLiquidityModal
            open={openRemoveLiquidityModal}
            onClose={handleCloseRemoveModal}
            row={selectedRow}
            key={openRemoveLiquidityModal ? 'open' : 'closed'}
          />
        )}
      </Card>
    </WalletGuard>
  );
}
