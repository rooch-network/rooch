import type { SessionInfoView } from '@roochnetwork/rooch-sdk';

import dayjs from 'dayjs';
import { useMemo, Fragment, useState } from 'react';

import { LoadingButton } from '@mui/lab';
import { Chip, Stack, Button, TableRow, Collapse, TableCell } from '@mui/material';

import { isSessionExpired } from 'src/utils/common';

import { toast } from 'src/components/snackbar';

export default function SessionKeyRowItem({
  item,
  removeSession,
}: {
  item: SessionInfoView;
  removeSession: (authKey: string) => Promise<void>;
}) {
  const [openCollapse, setOpenCollapse] = useState(false);
  const [removing, setRemoving] = useState(false);

  const expired = useMemo(
    () => isSessionExpired(Number(item.lastActiveTime), item.maxInactiveInterval),
    [item.lastActiveTime, item.maxInactiveInterval]
  );

  return (
    <Fragment key={item.authenticationKey}>
      <TableRow>
        <TableCell>{item.appName}</TableCell>
        <TableCell>
          <Button
            onClick={() => {
              setOpenCollapse(!openCollapse);
            }}
            size="small"
          >
            Show Scope
          </Button>
        </TableCell>
        <TableCell className="text-sm">
          {dayjs.unix(Number(item.createTime)).format('MMM DD, YYYY HH:mm:ss')}
        </TableCell>
        <TableCell className="text-sm">
          {dayjs.unix(Number(item.createTime)).format('MMM DD, YYYY HH:mm:ss')}
        </TableCell>
        <TableCell align="center">{item.maxInactiveInterval}</TableCell>
        <TableCell align="center">
          <LoadingButton
            loading={removing}
            onClick={async () => {
              setRemoving(true);
              try {
                await removeSession(item.authenticationKey);
                toast.success('Remove success');
              } catch (error) {
                toast.error(String(error));
              } finally {
                setRemoving(false);
              }
            }}
          >
            {expired ? 'Expired (Clear)' : 'Disconnect'}
          </LoadingButton>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell sx={{ p: 0.5 }} style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={6}>
          <Collapse in={openCollapse} timeout="auto" unmountOnExit>
            <Stack sx={{ p: 0.5, m: 0.5 }} className="bg-gray-50 rounded" spacing={1}>
              {item.scopes.map((i) => (
                <Chip
                  key={i.toString()}
                  label={i}
                  sx={{
                    justifyContent: 'flex-start',
                  }}
                  className="text-left w-full"
                  size="small"
                  variant="soft"
                  color="info"
                />
              ))}
            </Stack>
          </Collapse>
        </TableCell>
      </TableRow>
    </Fragment>
  );
}
