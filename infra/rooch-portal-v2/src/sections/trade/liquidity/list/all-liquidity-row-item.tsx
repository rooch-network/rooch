import dayjs from 'dayjs';
import { WalletGuard } from '@roochnetwork/rooch-sdk-kit';

import { Box, Button, TableRow, TableCell, ListItemText } from '@mui/material';

export type AllLiquidityItemType = {
  id: string;
  createAt: number;
  x: {
    id: string;
    symbol: string;
    type: string;
  };
  y: {
    id: string;
    symbol: string;
    type: string;
  };
  lpTokenId: string;
  creator: string;
};

type RowItemProps = {
  row: AllLiquidityItemType;
  onOpenViewModal: (row: AllLiquidityItemType) => void;
};

export default function AllLiquidityRowItem({ row, onOpenViewModal }: RowItemProps) {
  return (
    <TableRow>
      <TableCell width="300px">
        <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
          <ListItemText primary={dayjs(Number(row.createAt)).format('MMMM DD, YYYY HH:mm:ss')} />
        </Box>
      </TableCell>

      <TableCell>
        <ListItemText primary={row.x.symbol} />
      </TableCell>

      <TableCell>
        <ListItemText primary={row.y.symbol} />
      </TableCell>

      <TableCell>
        <ListItemText primary={row.creator} />
      </TableCell>

      <TableCell align="right" sx={{ pr: 1 }}>
        <WalletGuard
          onClick={() => {
            onOpenViewModal(row);
          }}
        >
          <Button variant="outlined" size="small">
            view
          </Button>
        </WalletGuard>
      </TableCell>
    </TableRow>
  );
}
