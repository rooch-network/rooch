import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { useState } from 'react';
import BigNumber from 'bignumber.js';
import PuffLoader from 'react-spinners/PuffLoader';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { grey } from '@mui/material/colors';
import {
  Box,
  Card,
  Stack,
  Dialog,
  Button,
  Tooltip,
  TextField,
  Typography,
  DialogTitle,
  DialogContent,
  DialogActions,
  InputAdornment,
} from '@mui/material';

import { toDust, fromDust, formatNumber } from 'src/utils/number';

import { warning, secondary } from 'src/theme/core';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';

import { toast } from 'src/components/snackbar';

import { Iconify } from '../iconify';
import InscriptionCard from './inscription-card';

export default function ListDialog({
  listDialogOpen,
  floorPrice,
  tick,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  refreshList,
  close,
}: {
  listDialogOpen: boolean;
  floorPrice: string;
  tick: string;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  refreshList: () => Promise<void>;
  close: () => void;
}) {
  const { mutate: signAndExecuteTransaction, isPending } = useSignAndExecuteTransaction();

  const [listPrice, setListPrice] = useState('');
  const [listAmount, setListAmount] = useState('');

  return (
    <Dialog
      open={listDialogOpen}
      onClose={close}
      sx={{
        '& .MuiDialog-paper': {
          minWidth: {
            xs: '360px',
            sm: '360px',
            md: '600px',
            lg: '600px',
          },
        },
      }}
    >
      <DialogTitle>List Token</DialogTitle>

      <DialogContent>
        {fromCoinBalanceInfo && toCoinBalanceInfo && (
          <Card
            variant="outlined"
            sx={{
              p: 2,
            }}
          >
            <InscriptionCard
              isVerified
              tick={toCoinBalanceInfo.symbol.toUpperCase()}
              tokenBalance={formatNumber(
                fromDust(toCoinBalanceInfo.balance, toCoinBalanceInfo.decimals).toNumber()
              )}
            />
          </Card>
        )}

        <Typography sx={{ mt: 3, mb: 0.5 }}>Price</Typography>

        <TextField
          autoFocus
          fullWidth
          type="number"
          InputProps={{
            endAdornment: (
              <InputAdornment position="end">
                {fromCoinBalanceInfo.symbol} / {toCoinBalanceInfo.symbol}
              </InputAdornment>
            ),
          }}
          margin="dense"
          value={listPrice}
          onChange={(e) => {
            setListPrice(e.target.value);
          }}
        />
        <Typography sx={{ mt: 3, mb: 0.5 }}>Amount</Typography>

        <TextField
          autoFocus
          fullWidth
          type="number"
          margin="dense"
          value={listAmount}
          onChange={(e) => {
            setListAmount(e.target.value);
          }}
        />
        {floorPrice !== undefined && (
          <Stack
            sx={{
              mt: 0.5,
              cursor: 'pointer',
            }}
            direction="row"
            alignItems="center"
            onClick={() => {
              setListPrice(floorPrice.toString());
            }}
            spacing={0.5}
          >
            <PuffLoader speedMultiplier={0.875} color={warning.light} loading size={16} />
            <Typography
              sx={{
                color: grey[500],
                fontSize: '0.875rem',
              }}
            >
              Latest floor price:{' '}
              <span
                style={{
                  color: secondary.light,
                }}
              >
                {floorPrice}
              </span>{' '}
              {fromCoinBalanceInfo.symbol}/{toCoinBalanceInfo.symbol}
            </Typography>
          </Stack>
        )}
        {fromCoinBalanceInfo && toCoinBalanceInfo && (
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Typography
              sx={{
                mt: 1,
              }}
            >
              Total Price:{' '}
              <span
                style={{
                  fontWeight: 600,
                  fontSize: '1.25rem',
                  color: secondary.light,
                }}
              >
                {new BigNumber(listPrice).times(listAmount).isNaN()
                  ? '-'
                  : new BigNumber(listPrice).times(listAmount).toFixed(4)}
              </span>{' '}
              {fromCoinBalanceInfo.symbol}
            </Typography>
            <Typography
              sx={{
                color: grey[500],
                fontSize: '0.875rem',
                display: 'flex',
                alignItems: 'center',
              }}
            >
              Fee: 2%{' '}
              <Tooltip
                title={
                  <Stack
                    sx={{
                      fontSize: '0.75rem',
                    }}
                  >
                    <Box>50% Market Fee</Box>
                    <Box>25% Community Fee</Box>
                    <Box>12.5% Burn Fee (Easter egg)</Box>
                    <Box>12.5% Locked in Inscription</Box>
                  </Stack>
                }
              >
                <Iconify
                  icon="solar:question-circle-bold"
                  width={16}
                  sx={{
                    ml: 1,
                  }}
                />
              </Tooltip>
            </Typography>
          </Stack>
        )}
      </DialogContent>

      <DialogActions>
        <Button onClick={close} variant="outlined" color="inherit">
          Cancel
        </Button>
        <LoadingButton
          // loading={isPending || isLoadingUserInscription}
          disabled={
            new BigNumber(listPrice).times(listAmount || 0).isNaN() ||
            new BigNumber(listPrice).isNaN() ||
            new BigNumber(listPrice).isZero()
            // (listItem && Number(listItem.data.content.fields.amount) < 10000)
          }
          onClick={() => {
            if (!fromCoinBalanceInfo || !toCoinBalanceInfo) {
              return;
            }
            const tx = new Transaction();

            tx.callFunction({
              target: `${TESTNET_ORDERBOOK_PACKAGE}::market_v2::list`,
              args: [
                Args.objectId('0x156d9a5bfa4329f999115b5febde94eed4a37cde10637ad8eed1ba91e89e0bb7'),
                Args.u256(
                  BigInt(
                    new BigNumber(
                      toDust(listAmount, toCoinBalanceInfo.decimals).toString()
                    ).toNumber()
                  )
                ),
                Args.u64(
                  BigInt(
                    new BigNumber(
                      toDust(listPrice, fromCoinBalanceInfo.decimals).toString()
                    ).toFixed()
                  )
                ),
              ],
              typeArgs: [
                '0x3::gas_coin::RGas',
                '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::fixed_supply_coin::FSC',
              ],
            });
            signAndExecuteTransaction(
              {
                transaction: tx,
              },
              {
                async onSuccess(data) {
                  toast.success('List success');
                  close();
                  refreshList();
                },
                onError(error) {
                  toast.error(String(error));
                },
              }
            );
          }}
          variant="contained"
        >
          Submit
        </LoadingButton>
      </DialogActions>
    </Dialog>
  );
}
