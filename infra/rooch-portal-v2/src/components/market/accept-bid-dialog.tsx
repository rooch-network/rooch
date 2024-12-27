import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import type { BidItem } from 'src/hooks/trade/use-market-data';

import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
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
  Typography,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material';

import { isMainNetwork } from 'src/utils/env';
import { fNumber } from 'src/utils/format-number';
import { formatUnitPrice } from 'src/utils/marketplace';
import { fromDust, formatNumber } from 'src/utils/number';

import { secondary } from 'src/theme/core';
import { NETWORK_PACKAGE } from 'src/config/trade';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import InscriptionShopCard from './inscription-shop-card';

export type AcceptBidDialogProps = {
  open: boolean;
  acceptBidItem: BidItem;
  tokenBalance: string;
  tick: string;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
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
  tick,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  tokenBalance,
  refreshBidList,
  close,
}: AcceptBidDialogProps) {
  const { mutate: signAndExecuteTransaction, isPending } = UseSignAndExecuteTransaction();

  const network = isMainNetwork() ? 'mainnet' : 'testnet';

  const price = useMemo(
    () =>
      new BigNumber(formatUnitPrice(acceptBidItem.unit_price, toCoinBalanceInfo.decimals))
        .times(acceptBidItem.quantity)
        .toString(),
    [acceptBidItem.quantity, acceptBidItem.unit_price, toCoinBalanceInfo.decimals]
  );

  const account = useCurrentAddress();

  return (
    <Dialog
      open={open}
      onClose={close}
      sx={{
        '& .MuiDialog-paper': {
          minWidth: {
            xs: '360px',
            sm: '360px',
            md: '480px',
            lg: '480px',
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
          }}
        >
          <InscriptionShopCard
            objectId={acceptBidItem.order_id}
            tick={tick}
            // isVerified={tick.toLowerCase() === 'move'}
            isVerified
            amount={acceptBidItem.quantity}
            price={price}
            unitPrice={formatUnitPrice(acceptBidItem.unit_price, toCoinBalanceInfo.decimals)}
            // acc={item.acc}
            fromCoinBalanceInfo={fromCoinBalanceInfo}
            toCoinBalanceInfo={toCoinBalanceInfo}
            seller={acceptBidItem.owner}
            selectMode={false}
            type="list"
          />
        </Card>

        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
          sx={{ mt: 3, mb: 0.25 }}
        >
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
                  {tick.toUpperCase()} Balance:{' '}
                  {fNumber(fromDust(tokenBalance, toCoinBalanceInfo.decimals).toNumber())}
                </Typography>
              </Stack>
            }
            size="small"
            variant="filled"
            color="secondary"
          />
        </Stack>

        <Typography sx={{ mt: 3, mb: 0.5 }}>
          You will receive:{' '}
          <span
            style={{
              fontWeight: 600,
              fontSize: '1.25rem',
              color: secondary.light,
            }}
          >
            {new BigNumber(price).isNaN()
              ? '--'
              : formatNumber(fromDust(price, fromCoinBalanceInfo.decimals).toNumber())}
          </span>{' '}
          {fromCoinBalanceInfo.symbol}
        </Typography>
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
          disabled={!account || new BigNumber(acceptBidItem.quantity).isGreaterThan(tokenBalance)}
          onClick={() => {
            if (!account || new BigNumber(acceptBidItem.quantity).isGreaterThan(tokenBalance)) {
              return;
            }
            console.log('🚀 ~ file: accept-bid-dialog.tsx:237 ~ acceptBidItem:', acceptBidItem);

            const tx = new Transaction();
            tx.callFunction({
              target: `${TESTNET_ORDERBOOK_PACKAGE}::market_v2::accept_bid`,
              args: [
                Args.objectId(NETWORK_PACKAGE[network].tickInfo[tick].MARKET_OBJECT_ID),
                Args.u64(BigInt(acceptBidItem.order_id)),
                Args.address(acceptBidItem.owner),
                Args.bool(true),
                Args.address(account.genRoochAddress().toStr()),
              ],
              typeArgs: ['0x3::gas_coin::RGas', toCoinBalanceInfo.coin_type],
            });

            signAndExecuteTransaction(
              {
                transaction: tx,
              },
              {
                async onSuccess(data) {
                  if (data.execution_info.status.type === 'executed') {
                    toast.success('Accept bid success');
                    close();
                    refreshBidList();
                  } else {
                    toast.error('Accept bid Failed');
                  }
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
