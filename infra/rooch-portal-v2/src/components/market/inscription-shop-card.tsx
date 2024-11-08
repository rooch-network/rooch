import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import Link from 'next/link';
import BigNumber from 'bignumber.js';
import { useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import { grey, yellow } from '@mui/material/colors';
import { Chip, Stack, Typography } from '@mui/material';

import { fromDust } from 'src/utils/number';
import { fNumber } from 'src/utils/format-number';

import { secondary } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

export interface InscriptionCardProps {
  objectId: string;
  tick: string;
  isVerified: boolean;
  amount: string;
  price: string;
  unitPrice: string;
  acc: string;
  type: 'list' | 'bid';
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  seller?: string;
  selectMode?: boolean;
}

export default function InscriptionShopCard({
  objectId,
  tick,
  isVerified,
  amount,
  price,
  unitPrice,
  acc,
  type = 'list',
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  seller,
  selectMode,
}: InscriptionCardProps) {
  const account = useCurrentAddress();

  return (
    <Stack
      justifyContent="center"
      alignItems="center"
      spacing={1}
      sx={{
        p: 1,
        borderRadius: '4px',
      }}
    >
      <Stack
        sx={{
          width: '100%',
        }}
        direction="row"
        alignItems="center"
        justifyContent="space-between"
      >
        <Chip
          size="small"
          label={
            <Stack
              direction="row"
              alignItems="center"
              sx={{
                fontSize: {
                  xs: '0.75rem',
                  sm: '0.8125rem',
                },
              }}
            >
              {tick.toUpperCase()}
              {isVerified && (
                <Iconify
                  icon="solar:verified-check-bold"
                  color={yellow.A200}
                  width={16}
                  sx={{
                    ml: 0.5,
                  }}
                />
              )}
            </Stack>
          }
          variant="filled"
          color="secondary"
        />
        {selectMode || type === 'bid' ? (
          <Chip
            size="small"
            label={
              seller === account?.toStr()
                ? '#Your Order'
                : type === 'bid'
                  ? '#BID ORDER'
                  : `#${objectId.slice(0, 8)}`
            }
            variant="soft"
            color="info"
            sx={{
              '& .MuiChip-label': {
                fontSize: {
                  xs: '0.75rem',
                  sm: '0.8125rem',
                },
              },
            }}
          />
        ) : (
          <Link href={`https://suivision.xyz/object/${objectId}`} target="_blank">
            <Chip
              size="small"
              label={
                seller === account?.genRoochAddress().toStr()
                  ? '#Your Order'
                  : `#${objectId.slice(objectId.length - 7, objectId.length)}`
              }
              variant="soft"
              color={seller === account?.genRoochAddress().toStr() ? 'success' : 'info'}
              sx={{
                '& .MuiChip-label': {
                  fontSize: {
                    xs: '0.75rem',
                    sm: '0.8125rem',
                  },
                },
              }}
            />
          </Link>
        )}
      </Stack>
      <Typography
        sx={{
          fontSize: '2rem',
          fontWeight: 600,
        }}
      >
        {fNumber(fromDust(amount, toCoinBalanceInfo.decimals).toNumber())}
      </Typography>

      <Typography
        sx={{
          fontWeight: '400',
          fontSize: '0.875rem',
          color: grey[600],
          display: 'flex',
          alignItems: 'center',
        }}
      >
        <Typography
          sx={{
            mr: 1,
            fontSize: '1rem',
            color: secondary.light,
          }}
        >
          {new BigNumber(unitPrice).isNaN()
            ? '--'
            : fromDust(unitPrice, fromCoinBalanceInfo.decimals).toPrecision(1)}
        </Typography>
        {fromCoinBalanceInfo.symbol.toUpperCase()}/{toCoinBalanceInfo.symbol.toUpperCase()}
      </Typography>
      {/* <Typography>{mrcItem.content.fields.tick}</Typography> */}

      <Typography
        sx={{
          fontWeight: 600,
          fontSize: '1.2rem',
          color: secondary.light,
        }}
      >
        {new BigNumber(price).isNaN()
          ? '--'
          : fromDust(price, fromCoinBalanceInfo.decimals).toPrecision(2)}{' '}
        {fromCoinBalanceInfo.symbol}
      </Typography>
      {/* <Typography
        sx={{
          fontWeight: '400',
          fontSize: '0.875rem',
          color: grey[600],
        }}
      >
        Seller: {fromDust(acc, 9).toFixed(5)}
      </Typography> */}
    </Stack>
  );
}
