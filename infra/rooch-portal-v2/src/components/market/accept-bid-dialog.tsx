import type { BidItemObject } from 'src/hooks/trade/use-market-data';
import type { InscriptionObject } from 'src/hooks/trade/use-address-owned-inscription';

import BigNumber from 'bignumber.js';
import { useMemo, useState } from 'react';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, UseSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { yellow } from '@mui/material/colors';
import {
  Card,
  Chip,
  Stack,
  Dialog,
  Button,
  TextField,
  Typography,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material';

import { fNumber } from 'src/utils/format-number';

import { secondary } from 'src/theme/core';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';
import { NETWORK, SUI_DECIMALS, NETWORK_PACKAGE } from 'src/config/trade';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import InscriptionShopCard from './inscription-shop-card';

export type AcceptBidDialogProps = {
  open: boolean;
  acceptBidItem: BidItemObject;
  userOwnedTickInscription: InscriptionObject[];
  tick: string;
  refreshBidList: () => Promise<void>;
  close: () => void;
};

// function makeAcceptBidTxb(
//   targetAmount: number,
//   userOwnedTickInscription: InscriptionObject[],
//   selectedBid: BidItemObject,
//   address: string,
//   tick: string
// ) {
//   const tx = new Transaction();
//   const sortedData = userOwnedTickInscription.sort(
//     (a, b) => Number(b.data.content.fields.amount) - Number(a.data.content.fields.amount)
//   );
//   const firstItem = sortedData[0];
//   const opSortedData = sortedData.length === 1 ? [sortedData[0]] : sortedData.slice(1);
//   let currentTotal = new BigNumber(firstItem.data.content.fields.amount);
//   let inputInscription:
//     | {
//         index: number;
//         resultIndex: number;
//         kind: 'NestedResult';
//       }
//     | undefined;
//   // eslint-disable-next-line no-restricted-syntax
//   for (const inscription of opSortedData) {
//     const inscriptionData = inscription.data.content.fields;
//     if (new BigNumber(currentTotal).isLessThanOrEqualTo(targetAmount)) {
//       tx.callFunction({
//         target: `${NETWORK_PACKAGE[NETWORK].MOVESCRIPTIONS_PACKAGE_ID}::movescription::merge`,
//         args: [Args.objectId(firstItem.data.objectId), Args.objectId(inscription.data.objectId)],
//       });
//       currentTotal = currentTotal.plus(inscriptionData.amount);

//       if (currentTotal.isEqualTo(targetAmount)) {
//         break;
//       }
//     } else {
//       const remainingAmt = new BigNumber(targetAmount).minus(currentTotal);
//       if (remainingAmt.isLessThan(0)) {
//         const [final] = tx.callFunction({
//           target: `${NETWORK_PACKAGE[NETWORK].MOVESCRIPTIONS_PACKAGE_ID}::movescription::do_split`,
//           arguments: [Args.objectId(firstItem.data.objectId), Args.u64(BigInt(targetAmount))],
//         });
//         inputInscription = final;
//         break;
//       }
//     }
//   }

//   txb.moveCall({
//     target: `${NETWORK_PACKAGE[NETWORK].MARKET_PACKAGE_ID}::market::accept_bid`,
//     arguments: [
//       txb.object(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
//       txb.object(inputInscription || firstItem.data.objectId),
//       txb.pure(selectedBid.bidder),
//       txb.pure(selectedBid.bidId),
//       txb.object('0x6'),
//     ],
//   });

//   if (inputInscription) {
//     txb.transferObjects([firstItem.data.objectId], address);
//   }

//   return txb;
// }

export default function AcceptBidDialog({
  open,
  acceptBidItem,
  userOwnedTickInscription,
  tick,
  refreshBidList,
  close,
}: AcceptBidDialogProps) {
  const { mutate: signAndExecuteTransaction, isPending } = UseSignAndExecuteTransaction();

  const account = useCurrentAddress();

  const [acceptAmount, setAcceptAmount] = useState('');

  const inscriptionBalance = useMemo(() => {
    let total = new BigNumber(0);
    userOwnedTickInscription.forEach((i) => {
      total = total.plus(i.data.content.fields.amount);
    });
    return total;
  }, [userOwnedTickInscription, userOwnedTickInscription?.length]);

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
      <DialogTitle>Accept Bid</DialogTitle>

      <DialogContent>
        <Card
          variant="outlined"
          sx={{
            p: 2,
            cursor: 'pointer',
          }}
          onClick={() => {
            setAcceptAmount(acceptBidItem.amt);
          }}
        >
          <InscriptionShopCard
            objectId={acceptBidItem.bidId}
            tick={tick}
            isVerified
            amount={acceptBidItem.amt}
            price={acceptBidItem.price}
            unitPrice={new BigNumber(acceptBidItem.price).div(acceptBidItem.amt).toString()}
            acc="0"
            selectMode={false}
            type="bid"
          />
        </Card>

        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
          sx={{ mt: 3, mb: 0.25 }}
        >
          <Typography>Accept Amount</Typography>
          <Chip
            label={
              <Stack direction="row" alignItems="center">
                <Iconify
                  icon="solar:wallet-bold"
                  color={yellow.A200}
                  width={18}
                  sx={{
                    mr: 0.5,
                  }}
                />
                <Typography
                  sx={{
                    fontWeight: 600,
                    fontSize: '0.875rem',
                  }}
                >
                  ${tick.toUpperCase()}: {fNumber(inscriptionBalance.toNumber())}
                </Typography>
              </Stack>
            }
            size="small"
            variant="filled"
            color="secondary"
          />
        </Stack>
        <TextField
          autoFocus
          fullWidth
          type="number"
          InputProps={
            {
              // endAdornment: (
              //   <InputAdornment position="end">
              //     SUI / {listItem?.data.content.fields.tick}
              //   </InputAdornment>
              // ),
            }
          }
          margin="dense"
          value={acceptAmount}
          onChange={(e) => {
            setAcceptAmount(e.target.value);
          }}
        />

        <Typography sx={{ mt: 3, mb: 0.5 }}>
          You will receive:{' '}
          <span
            style={{
              fontWeight: 600,
              fontSize: '1.25rem',
              color: secondary.light,
            }}
          >
            {new BigNumber(acceptAmount).isNaN()
              ? '--'
              : new BigNumber(acceptAmount)
                  .times(acceptBidItem.unitPrice)
                  .div(new BigNumber(10).pow(SUI_DECIMALS))
                  .toFixed(4)}
          </span>{' '}
          SUI
        </Typography>

        {/* {floorPrice !== undefined && (
          <Stack
            sx={{
              mt: 0.5,
              cursor: 'pointer',
            }}
            direction="row"
            alignItems="center"
            onClick={() => {
              setBatchListPrice(floorPrice.toString());
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
              SUI/{currentTab.toUpperCase()}
            </Typography>
          </Stack>
        )} */}

        {/* <Typography
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
            {new BigNumber(batchListPrice).times(batchAmount).isNaN()
              ? '-'
              : new BigNumber(batchListPrice).times(batchAmount).toFixed(4)}
          </span>{' '}
          SUI
        </Typography> */}
      </DialogContent>

      <DialogActions>
        <Button
          onClick={() => {
            close();
          }}
          variant="outlined"
          color="inherit"
        >
          Cancel
        </Button>
        <LoadingButton
          loading={isPending}
          disabled={
            !account ||
            new BigNumber(acceptAmount).isNaN() ||
            new BigNumber(acceptAmount).isZero() ||
            new BigNumber(acceptAmount).isGreaterThan(inscriptionBalance)
          }
          onClick={() => {
            if (
              !account ||
              new BigNumber(acceptAmount).isNaN() ||
              new BigNumber(acceptAmount).isZero() ||
              new BigNumber(acceptAmount).isGreaterThan(inscriptionBalance)
            ) {
              return;
            }

            const tx = new Transaction();
            tx.callFunction({
              target: `${TESTNET_ORDERBOOK_PACKAGE}::market::create_bid`,
              args: [
                Args.objectId(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
                Args.u64(BigInt(acceptBidItem.bidId)),
                Args.bool(true),
                Args.address(TESTNET_ORDERBOOK_PACKAGE),
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
                  toast.success('Accept bid success');
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
