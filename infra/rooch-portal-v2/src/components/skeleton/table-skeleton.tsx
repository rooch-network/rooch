import type { TableCellProps } from '@mui/material';

import Skeleton from '@mui/material/Skeleton';
import { TableRow, TableCell } from '@mui/material';

export default function TableSkeleton({
  col,
  row,
  rowHeight,
}: {
  col: number;
  row: number;
  rowHeight?: TableCellProps['height'];
}) {
  return Array.from({ length: row }).map((_, index) => (
    <TableRow key={index}>
      {Array.from({ length: col }).map((__, jIndex) => (
        <TableCell
          key={jIndex}
          height={rowHeight}
          width={jIndex === 0 ? '256px' : undefined}
          align={jIndex === 5 ? 'center' : undefined}
        >
          <Skeleton sx={{ width: '100%', height: 16 }} />
        </TableCell>
      ))}
    </TableRow>
  ));
}
