import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { WalletGuard } from '@roochnetwork/rooch-sdk-kit';

import { Box, Button, TableRow, TableCell, ListItemText } from '@mui/material';

export type OwnerLiquidityItemType = {
  x: {
    name: string;
    type: string;
  };
  y: {
    name: string;
    type: string;
  };
} & BalanceInfoView;

type RowItemProps = {
  row: OwnerLiquidityItemType;
  onOpenViewModal: (row: OwnerLiquidityItemType) => void;
};

export default function OwnerLiquidityRowItem({ row, onOpenViewModal }: RowItemProps) {
  // const coin = useMemo(() => {
  //   const t = row.coin_type.split(',');
  //   const x = t[0];
  //   const y = t[1];
  //   const xName = x.split('::');
  //   const yName = y.split('::');
  //   return {
  //     x: {
  //       type: x,
  //       name: xName[xName.length - 1].replaceAll('>', ''),
  //     },
  //     y: {
  //       type: y,
  //       name: yName[yName.length - 1].replaceAll('>', ''),
  //     },
  //   };
  // }, [row]);
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
            view
          </Button>
        </WalletGuard>
      </TableCell>
    </TableRow>
  );
}
