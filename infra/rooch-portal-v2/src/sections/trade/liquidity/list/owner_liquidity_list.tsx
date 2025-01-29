import { useState } from 'react';

import { Card, Table, TableBody } from '@mui/material';

import { Scrollbar } from 'src/components/scrollbar';
import WalletGuard from 'src/components/guard/WalletGuard';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import RemoveLiquidityModal from './remove-liquidity-modal';
import OwnerLiquidityRowItem from './owner-liquidity-row-item';
import { useOwnerLiquidity } from '../../hooks/use-owner-liquidity';

import type { OwnerLiquidityItemType } from '../../hooks/use-owner-liquidity';

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
  const [selectedRow, setSelectedRow] = useState<OwnerLiquidityItemType>();

  const handleRemoveModal = (row: OwnerLiquidityItemType) => {
    setSelectedRow(row);
    setOpenRemoveLiquidityModal(true);
  };

  const handleCloseRemoveModal = () => {
    setOpenRemoveLiquidityModal(false);
    setSelectedRow(undefined);
  };

  return (
    <WalletGuard>
      <Card className="mt-4">
        {/* <Box
          display="flex"
          justifyContent="space-between"
          alignItems="center"
          sx={{ ml: 2, mr: 1, height: 60 }}
        >
          <Box display="flex" width="100%" justifyContent="flex-end" alignItems="center">
            <Link href="./pool/add">
              <Button variant="outlined">Create Liquidity</Button>
            </Link>
          </Box>
        </Box> */}
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
                      onOpenViewModal={handleRemoveModal}
                    />
                  ))}
                  <TableNoData title="No Coins Found" notFound={lpTokens?.length === 0} />
                </>
              )}
            </TableBody>
          </Table>
        </Scrollbar>

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
