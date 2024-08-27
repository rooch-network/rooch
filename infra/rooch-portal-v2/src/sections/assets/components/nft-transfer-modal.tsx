import type { RefetchOptions, QueryObserverResult } from '@tanstack/react-query';
import type {
  IndexerObjectStateView,
  PaginatedIndexerObjectStateViews,
} from '@roochnetwork/rooch-sdk';

import { useState } from 'react';
import { useCurrentSession, useTransferObject } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Box,
  Stack,
  Button,
  Dialog,
  TextField,
  DialogTitle,
  FormControl,
  DialogActions,
  DialogContent,
  FormHelperText,
} from '@mui/material';

import { toast } from 'src/components/snackbar';

export default function NFTTransferModal({
  open,
  onClose,
  selectedNFT,
  refetch,
}: {
  open: boolean;
  onClose: () => void;
  selectedNFT: IndexerObjectStateView;
  refetch: (
    options?: RefetchOptions
  ) => Promise<QueryObserverResult<PaginatedIndexerObjectStateViews, Error>>;
}) {
  const sessionKey = useCurrentSession();
  const [recipient, setRecipient] = useState('');

  const [transferring, setTransferring] = useState(false);
  const { mutateAsync: transferObject } = useTransferObject();

  return (
    <Dialog open={open}>
      <DialogTitle sx={{ pb: 2 }}>NFT Transfer</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          overflow: 'unset',
        }}
      >
        <Stack direction="column" className="mb-2 w-full" spacing={1}>
          <Box className="font-semibold">{selectedNFT.display_fields?.fields.name as string}</Box>
          <Box
            component="img"
            className="aspect-square rounded-xl"
            src={`data:image/svg+xml;base64,${btoa(selectedNFT.display_fields?.fields.image_url as string)}`}
          />
        </Stack>
        <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1 }}>
          <FormControl>
            <TextField
              label="Recipient"
              inputMode="text"
              spellCheck="false"
              rows={2}
              multiline
              autoComplete="off"
              InputProps={{
                spellCheck: 'false',
              }}
              value={recipient}
              onChange={(e) => {
                setRecipient(e.target.value);
              }}
            />
          </FormControl>
        </Stack>

        {false && (
          <FormHelperText error sx={{ px: 2 }}>
            invalid value
          </FormHelperText>
        )}
      </DialogContent>

      <DialogActions>
        <Button
          fullWidth
          variant="outlined"
          color="inherit"
          onClick={() => {
            onClose();
          }}
        >
          Cancel
        </Button>

        <LoadingButton
          fullWidth
          loading={transferring}
          disabled={false}
          variant="contained"
          onClick={async () => {
            try {
              if (!selectedNFT || recipient === '' || !sessionKey) {
                return;
              }
              setTransferring(true);
              await transferObject({
                signer: sessionKey!,
                recipient,
                objectId: selectedNFT.id,
                objectType: {
                  target: selectedNFT.object_type,
                },
              });
              onClose();
              refetch();
              toast.success('Transfer success');
            } catch (error) {
              toast.error(String(error));
            } finally {
              setTransferring(false);
            }
          }}
        >
          Confirm
        </LoadingButton>
      </DialogActions>
    </Dialog>
  );
}
