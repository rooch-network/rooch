import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { Box, Button, TableRow, TableCell, ListItemText } from '@mui/material';

import { formatCoin } from 'src/utils/format-number';

type RowItemProps = {
  row: BalanceInfoView;
  isWalletOwner: boolean;
  onOpenTransferModal: (row: BalanceInfoView) => void;
};

export default function AssetRowItem({ row, isWalletOwner, onOpenTransferModal }: RowItemProps) {
  return (
    <TableRow>
      <TableCell width="300px">
        <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
          <ListItemText primary={row.symbol} secondary={row.name} />
        </Box>
      </TableCell>

      <TableCell>
        <ListItemText
          primary={formatCoin(Number(row.balance), row.decimals, row.decimals)}
          primaryTypographyProps={{
            typography: 'body2',
            sx: {
              fontWeight: 600,
            },
          }}
          secondaryTypographyProps={{ mt: 0.5, component: 'span', typography: 'caption' }}
        />
      </TableCell>

      {isWalletOwner && (
        <TableCell align="right" sx={{ pr: 1 }}>
          <Button
            variant="outlined"
            size="small"
            onClick={() => {
              onOpenTransferModal(row);
            }}
          >
            Transfer
          </Button>
        </TableCell>
      )}
    </TableRow>
  );
}
