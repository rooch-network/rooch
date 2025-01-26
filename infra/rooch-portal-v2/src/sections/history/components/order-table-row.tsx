import Link from 'next/link';
import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
import dayjs from 'dayjs/esm/index.js';
import relativeTime from 'dayjs/esm/plugin/relativeTime';

import Stack from '@mui/material/Stack';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import { Chip, Typography } from '@mui/material';
import ListItemText from '@mui/material/ListItemText';

import { shortAddress } from 'src/utils/address';
import { fNumber } from 'src/utils/format-number';
import { formatUnitPrice } from 'src/utils/marketplace';
import { fromDust, formatNumber } from 'src/utils/number';

import { grey, secondary } from 'src/theme/core/palette';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { EVENT_ENUM, ORDER_TYPE_MAP } from '../utils/common';

import type { RenderEvent } from '../utils/types';

type Props = {
  row: RenderEvent;
  selected: boolean;
  tickInfo: {
    tick: string;
    types: string;
    fromTokenDecimals: bigint;
    fromTokenSymbol: string;
    toTokenDecimals: bigint;
    toTokenSymbol: string;
  };
  onViewRow: VoidFunction;
  onSelectRow: VoidFunction;
  onDeleteRow: VoidFunction;
};

dayjs.extend(relativeTime);

function NoDataCell() {
  return (
    <span
      style={{
        color: grey[600],
      }}
    >
      --
    </span>
  );
}

export default function OrderTableRow({
  row,
  selected,
  tickInfo,
  onViewRow,
  onSelectRow,
  onDeleteRow,
}: Props) {
  const { order_type, sender, quantity, timestamp, unit_price, tx_hash } = row;
  console.log('🚀 ~ file: order-table-row.tsx:64 ~ unit_price:', unit_price);

  const type = ORDER_TYPE_MAP[order_type];

  const price = useMemo(
    () =>
      new BigNumber(formatUnitPrice(unit_price, Number(tickInfo.fromTokenDecimals.toString())))
        .times(fromDust(quantity, tickInfo.fromTokenDecimals))
        .toString(),
    [tickInfo.fromTokenDecimals, quantity, unit_price]
  );

  const renderPrimary = (
    <TableRow hover selected={selected}>
      {/* <TableCell padding="checkbox">
        <Checkbox checked={selected} onClick={onSelectRow} />
      </TableCell> */}

      <TableCell>
        <Label
          variant="outlined"
          color={
            ((type === EVENT_ENUM.BuyEvent || type === EVENT_ENUM.AcceptBidEvent) && 'success') ||
            ((type === EVENT_ENUM.ListEvent || type === EVENT_ENUM.CreateBidEvent) && 'warning') ||
            (type === EVENT_ENUM.CancelListEvent && 'primary') ||
            'default'
          }
        >
          {type}
        </Label>
      </TableCell>

      <TableCell>
        <Chip
          size="small"
          label={
            <Stack
              direction="row"
              alignItems="center"
              sx={{
                color: grey[800],
              }}
            >
              {tickInfo.tick || '--'}
              <Iconify
                icon="solar:verified-check-bold"
                color={secondary.main}
                width={16}
                sx={{
                  ml: 0.5,
                }}
              />
            </Stack>
          }
          variant="outlined"
          color="secondary"
        />
      </TableCell>

      <TableCell
        sx={{
          fontWeight: 600,
        }}
      >
        {type === EVENT_ENUM.CancelListEvent ? <NoDataCell /> : <>{fNumber(quantity)}</>}
      </TableCell>

      {/* Price */}
      <TableCell>
        <ListItemText
          primary={
            <Typography
              sx={{
                fontSize: '0.875rem',
                fontWeight: 600,
                color: secondary.light,
              }}
            >
              {fromDust(price, tickInfo.fromTokenDecimals).toFixed(4)}{' '}
              {tickInfo.fromTokenSymbol.toUpperCase()}{' '}
            </Typography>
          }
          secondary={
            type === EVENT_ENUM.CancelListEvent ? (
              <NoDataCell />
            ) : (
              <span>
                {formatNumber(
                  fromDust(
                    formatUnitPrice(unit_price, Number(tickInfo.toTokenDecimals.toString())),
                    tickInfo.fromTokenDecimals
                  ).toNumber()
                )}{' '}
                {tickInfo.tick || '--'}
              </span>
            )
          }
          primaryTypographyProps={{ typography: 'body2', noWrap: true }}
          secondaryTypographyProps={{
            mt: 0.5,
            component: 'span',
            typography: 'caption',
          }}
        />
      </TableCell>

      <TableCell>{shortAddress(sender) || <NoDataCell />}</TableCell>

      {/* <TableCell>{shortAddress(owner) || <NoDataCell />}</TableCell> */}

      {/* <TableCell> {shortAddress(owner) || <NoDataCell />} </TableCell> */}

      <TableCell
        sx={{
          fontSize: '0.875rem',
          // color: grey[600],
        }}
      >
        <ListItemText
          primary={
            <Typography
              sx={{
                fontSize: '0.875rem',
              }}
            >
              {dayjs(Number(timestamp)).fromNow()}
            </Typography>
          }
          secondary={
            <Stack direction="row" alignItems="center">
              <Link
                href={`https://roochscan.io/main/tx/${tx_hash}`}
                target="_blank"
                style={{
                  color: grey[600],
                  textDecoration: 'none',
                  display: 'flex',
                  alignItems: 'center',
                }}
              >
                {shortAddress(tx_hash)}
                <Iconify
                  icon="solar:square-forward-broken"
                  width={14}
                  sx={{
                    ml: 0.5,
                  }}
                />
              </Link>
            </Stack>
          }
          primaryTypographyProps={{ typography: 'body2', noWrap: true }}
          secondaryTypographyProps={{
            mt: 1,
            component: 'span',
            typography: 'caption',
          }}
        />
      </TableCell>
    </TableRow>
  );

  return <>{renderPrimary}</>;
}
