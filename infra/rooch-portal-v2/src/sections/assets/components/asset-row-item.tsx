import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import DOMPurify from 'dompurify';

import { Box, Button, TableRow, TableCell, ListItemText } from '@mui/material';

import { formatCoin } from 'src/utils/format-number';

import { Iconify } from 'src/components/iconify';

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
          {/* {row.icon_url && <Image src={row.icon_url} alt={row.symbol} width={48} height={48} />} */}
          {row.icon_url ? (
            <Box
              component="span"
              className="mr-1"
              sx={{ width: 32, height: 32 }}
              dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(row.icon_url) }}
            />
          ) : (
            <Iconify
              className="mr-1"
              icon="solar:question-circle-line-duotone"
              width={32}
              height={32}
            />
          )}
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
