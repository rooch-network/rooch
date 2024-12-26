import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { useState } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, UseSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Card,
  Stack,
  Button,
  Dialog,
  TextField,
  Typography,
  DialogTitle,
  DialogActions,
  DialogContent,
  InputAdornment,
} from '@mui/material';

import { toDust } from 'src/utils/number';
import { isMainNetwork } from 'src/utils/env';

import { secondary } from 'src/theme/core';
import { NETWORK_PACKAGE } from 'src/config/trade';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';

import { toast } from '../snackbar';
import InscriptionShopCard from './inscription-shop-card';

export type CreateBidDialogProps = {
  open: boolean;
  tick: string;
  floorPrice: number;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  refreshBidList: () => Promise<void>;
  close: () => void;
};

export default function CreateBidDialog({
  open,
  tick,
  floorPrice,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  refreshBidList,
  close,
}: CreateBidDialogProps) {
  const network = isMainNetwork() ? 'mainnet' : 'testnet';
  const [bidAmount, setBidAmount] = useState('');
  const [bidUnitPrice, setBidUnitPrice] = useState('');

  const account = useCurrentAddress();
  const { mutate: signAndExecuteTransaction, isPending } = UseSignAndExecuteTransaction();

  return (
    <Dialog
      open={open}
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
      <DialogTitle>Create Bid</DialogTitle>

      <DialogContent>
        <Card
          variant="outlined"
          sx={{
            p: 2,
          }}
        >
          <InscriptionShopCard
            objectId="#"
            tick={tick}
            isVerified={tick.toLowerCase() === 'move'}
            amount={toDust(bidAmount, toCoinBalanceInfo.decimals).toString()}
            price={new BigNumber(bidAmount)
              .times(bidUnitPrice)
              .times(new BigNumber(10).pow(fromCoinBalanceInfo.decimals))
              .toString()}
            unitPrice={new BigNumber(bidUnitPrice)
              .times(new BigNumber(10).pow(fromCoinBalanceInfo.decimals))
              .toString()}
            selectMode={false}
            type="bid"
            fromCoinBalanceInfo={fromCoinBalanceInfo}
            toCoinBalanceInfo={toCoinBalanceInfo}
          />
        </Card>

        <Typography sx={{ mt: 3, mb: 0.5 }}>Bid Amount</Typography>

        <TextField
          autoFocus
          fullWidth
          autoComplete="off"
          type="number"
          InputProps={{
            endAdornment: (
              <InputAdornment position="end">{toCoinBalanceInfo.symbol}</InputAdornment>
            ),
          }}
          margin="dense"
          value={bidAmount}
          onChange={(e) => {
            const { value } = e.target;
            const parts = value.split('.');
            if (parts.length === 2 && parts[1].length > toCoinBalanceInfo.decimals) {
              return;
            }
            setBidAmount(value);
          }}
        />

        <Typography sx={{ mt: 3, mb: 0.5 }}>Bid Unit Price</Typography>
        <TextField
          autoFocus
          fullWidth
          autoComplete="off"
          type="number"
          InputProps={{
            endAdornment: (
              <InputAdornment position="end">
                {fromCoinBalanceInfo.symbol} / {toCoinBalanceInfo.symbol}
              </InputAdornment>
            ),
          }}
          margin="dense"
          value={bidUnitPrice}
          onChange={(e) => {
            const { value } = e.target;
            const parts = value.split('.');
            if (parts.length === 2 && parts[1].length > fromCoinBalanceInfo.decimals) {
              return;
            }
            setBidUnitPrice(value);
          }}
        />
        {/* <Stack
          sx={{
            mt: 0.5,
            cursor: 'pointer',
          }}
          direction="row"
          alignItems="center"
          onClick={() => {
            setBidUnitPrice(floorPrice.toString());
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
            Latest Floor Price:{' '}
            <span
              style={{
                color: secondary.light,
              }}
            >
              {floorPrice}
            </span>{' '}
            SUI/{tick.toUpperCase()}
          </Typography>
        </Stack> */}
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
              {new BigNumber(bidAmount).times(bidUnitPrice).isNaN()
                ? '-'
                : new BigNumber(bidAmount).times(bidUnitPrice).toFixed(4)}
            </span>{' '}
            {fromCoinBalanceInfo.symbol}
          </Typography>
          {/* <Typography
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
          </Typography> */}
        </Stack>
      </DialogContent>

      <DialogActions>
        <Button onClick={close} variant="outlined" color="inherit">
          Cancel
        </Button>
        <LoadingButton
          loading={isPending}
          disabled={
            new BigNumber(bidAmount).isNaN() ||
            new BigNumber(bidUnitPrice).isNaN() ||
            new BigNumber(bidAmount).isZero() ||
            new BigNumber(bidUnitPrice).isZero()
          }
          onClick={() => {
            if (!account) {
              return;
            }
            const tx = new Transaction();

            const unitPriceInMist = toDust(bidUnitPrice, fromCoinBalanceInfo.decimals);

            tx.callFunction({
              target: `${TESTNET_ORDERBOOK_PACKAGE}::market_v2::create_bid`,
              args: [
                Args.objectId(NETWORK_PACKAGE[network].tickInfo[tick].MARKET_OBJECT_ID),
                Args.u64(unitPriceInMist),
                Args.u256(BigInt(toDust(bidAmount, toCoinBalanceInfo.decimals))),
              ],
              typeArgs: ['0x3::gas_coin::RGas', toCoinBalanceInfo.coin_type],
            });

            signAndExecuteTransaction(
              {
                transaction: tx,
              },
              {
                async onSuccess(data) {
                  // await refetchAddressOwnedInscription();
                  toast.success('Create Bid Success');
                  close();
                  refreshBidList();
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
