import { WalletGuard } from '@roochnetwork/rooch-sdk-kit';

import { Box, Button, TableRow, TableCell, ListItemText } from '@mui/material';

import { fromDust, formatByIntl } from 'src/utils/number';

import type { OwnerLiquidityItemType } from 'src/sections/trade/hooks/use-owner-liquidity';

type RowItemProps = {
  row: OwnerLiquidityItemType;
  onOpenRemoveModal: (row: OwnerLiquidityItemType) => void;
  onOpenAddModal: (row: OwnerLiquidityItemType) => void;
};

export default function OwnerLiquidityRowItem({
  row,
  onOpenAddModal,
  onOpenRemoveModal,
}: RowItemProps) {
  console.log(row.fixedBalance);
  return (
    <TableRow>
      <TableCell width="300px">
        <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
          <ListItemText primary={row.name} />
        </Box>
      </TableCell>
      <TableCell>
        <ListItemText primary={row.x.name} />
      </TableCell>
      <TableCell>
        <ListItemText primary={row.y.name} />
      </TableCell>
      <TableCell>
        <ListItemText primary={formatByIntl(row.fixedBalance)} />
      </TableCell>
      <TableCell>
        <ListItemText primary={formatByIntl(fromDust(row.supply, row.decimals).toString())} />
      </TableCell>

      <TableCell align="right" sx={{ pr: 1 }}>
        {row.fixedBalance > 0 && (
          <WalletGuard
            onClick={() => {
              onOpenRemoveModal(row);
            }}
          >
            <Button variant="outlined" size="small">
              Remove
            </Button>
          </WalletGuard>
        )}
        <WalletGuard
          onClick={() => {
            onOpenAddModal(row);
          }}
        >
          <Button sx={{ ml: 1 }} variant="outlined" size="small">
            Add
          </Button>
        </WalletGuard>
      </TableCell>
    </TableRow>
  );
}
