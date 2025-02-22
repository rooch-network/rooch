import { useState } from 'react';

import { Card, Table, Stack, TableBody, Pagination } from '@mui/material';

import useTokenPairInfos from 'src/hooks/liquidity/use-token-pair-infos';

import { Scrollbar } from 'src/components/scrollbar';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import AllLiquidityRowItem from './add-liquidity-modal';
import LiquidityRowItem from './all-liquidity-row-item';
import { useAllLiquidity } from '../../hooks/use-all-liquidity';

import type { AllLiquidityItemType } from '../../hooks/use-all-liquidity';

const headerLabel = [
  { id: 'create_at', label: 'Create At' },
  { id: 'x', label: 'X' },
  { id: 'y', label: 'Y' },
  { id: 'action', label: 'Action', align: 'right' },
];

export default function AllLiquidityList() {
  const { isPending, lpTokens, hasNext, index, paginate } = useAllLiquidity();

  const { tokenPairInfos, isPending: isTokenPairInfosPending } = useTokenPairInfos(lpTokens);

  const [openAddLiquidityModal, setOpenAddLiquidityModal] = useState(false);
  const [selectedRow, setSelectedRow] = useState<AllLiquidityItemType>();

  const handleRemoveModal = (row: AllLiquidityItemType) => {
    setSelectedRow(row);
    setOpenAddLiquidityModal(true);
  };

  const handleCloseRemoveModal = () => {
    setOpenAddLiquidityModal(false);
    setSelectedRow(undefined);
  };

  return (
    <Card className="mt-4">
      <Scrollbar sx={{ minHeight: 462 }}>
        <Table sx={{ minWidth: 720 }} size="medium">
          <TableHeadCustom headLabel={headerLabel} />

          <TableBody>
            {isPending || isTokenPairInfosPending ? (
              <TableSkeleton col={headerLabel.length} row={5} rowHeight="77px" />
            ) : (
              <>
                {lpTokens.map((row) => (
                  <LiquidityRowItem
                    key={row.id}
                    row={row}
                    balance={tokenPairInfos?.get(row.id)}
                    onOpenViewModal={handleRemoveModal}
                  />
                ))}
                <TableNoData title="No Coins Found" notFound={lpTokens.length === 0} />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
      {(hasNext || index > 1) && (
        <Stack className="mb-4 w-full mt-4" alignItems="flex-end">
          <Pagination
            count={hasNext ? index + 1 : index}
            page={index}
            onChange={(event: React.ChangeEvent<unknown>, value: number) => {
              paginate(value);
            }}
          />
        </Stack>
      )}
      {selectedRow && (
        <AllLiquidityRowItem
          open={openAddLiquidityModal}
          onClose={handleCloseRemoveModal}
          row={selectedRow}
          key={openAddLiquidityModal ? 'open' : 'closed'}
        />
      )}
    </Card>
  );
}
