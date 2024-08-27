import type { Theme, SxProps } from '@mui/material/styles';

import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';

import { EmptyContent } from '../empty-content';

export type TableNoDataProps = {
  notFound: boolean;
  title?: string;
  sx?: SxProps<Theme>;
};

export function TableNoData({ notFound, title, sx }: TableNoDataProps) {
  return (
    <TableRow>
      {notFound && (
        <TableCell colSpan={12}>
          <EmptyContent title={title} sx={{ py: 10, ...sx }} />
        </TableCell>
      )}
    </TableRow>
  );
}
