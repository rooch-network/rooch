import {
  Box,
  Stack,
  Button,
  TableRow,
  TableCell,
  ListItemText,
  LinearProgress,
} from '@mui/material';

import type { MintType } from './mint-table-card';

type RowItemProps = {
  row: MintType;
  isStaticData?: boolean;
};

export default function MintRowItem({ row, isStaticData }: RowItemProps) {
  return (
    <TableRow>
      <TableCell>
        <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
          <ListItemText primary={row.symbol} />
        </Box>
      </TableCell>

      <TableCell>
        <ListItemText primary={row.name} />
      </TableCell>

      <TableCell>{row.distribution}</TableCell>

      <TableCell width={300}>
        <Stack direction="row" alignItems="center" spacing={2}>
          <LinearProgress
            color="success"
            sx={{
              height: '6px',
            }}
            variant="determinate"
            value={row.progress === -1 ? 100 : row.progress}
            className="w-[60%]"
          />
          <Box>{row.progress === -1 ? '∞' : `${row.progress}%`}</Box>
        </Stack>
      </TableCell>

      {isStaticData && (
        <TableCell align="right">
          <Button disabled={isStaticData} size="small" variant="outlined">
            {isStaticData ? 'Coming Soon' : 'Mint'}
          </Button>
        </TableCell>
      )}
    </TableRow>
  );
}
