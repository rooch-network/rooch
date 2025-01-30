
import { WalletGuard } from '@roochnetwork/rooch-sdk-kit';

import { Box, Button, TableRow, TableCell, ListItemText } from '@mui/material';

import type { OwnerLiquidityItemType } from '../../hooks/use-owner-liquidity';

type RowItemProps = {
  row: OwnerLiquidityItemType;
  onOpenViewModal: (row: OwnerLiquidityItemType) => void;
};

export default function OwnerLiquidityRowItem({ row, onOpenViewModal }: RowItemProps) {
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
        <ListItemText primary={row.fixedBalance} />
      </TableCell>
      <TableCell>
        <ListItemText primary={row.supply} />
      </TableCell>

      <TableCell align="right" sx={{ pr: 1 }}>
        <WalletGuard
          onClick={() => {
            onOpenViewModal(row);
          }}
        >
          <Button variant="outlined" size="small">
            Remove
          </Button>
        </WalletGuard>
      </TableCell>
    </TableRow>
  );
}
