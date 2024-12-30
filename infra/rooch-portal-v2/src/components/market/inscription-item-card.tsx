import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import type { MarketItem } from 'src/hooks/trade/use-market-data';

import { useState, useMemo } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { grey } from '@mui/material/colors';
import { Dialog, DialogActions, DialogContent, Button, Card, Chip, Stack, Checkbox, CardActions } from '@mui/material';

import { formatNumber, fromDust } from 'src/utils/number';
import { formatUnitPrice } from 'src/utils/marketplace';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import InscriptionShopCard from './inscription-shop-card';
import { useNetworkVariable } from '../../hooks/use-networks';

export type InscriptionItemCardProps = {
  item: MarketItem;
  tick: string;
  accountBalance?: string;
  selectMode?: boolean;
  selected?: boolean;
  fromCoinBalanceInfo: BalanceInfoView;
  toCoinBalanceInfo: BalanceInfoView;
  onSelectItem: (inputValue: string) => void;
  onRefetchMarketData: () => Promise<void>;
};

export default function InscriptionItemCard({
  item,
  tick,
  accountBalance,
  selectMode,
  selected,
  fromCoinBalanceInfo,
  toCoinBalanceInfo,
  onSelectItem,
  onRefetchMarketData,
}: InscriptionItemCardProps) {
  const account = useCurrentAddress();
  const market = useNetworkVariable('market');
  const { mutate: signAndExecuteTransaction, isPending } = useSignAndExecuteTransaction();
  const [openDialog, setOpenDialog] = useState(false);
  const [confirmData, setConfirmData] = useState({
    price: '0',
    quantity: 0,
  });

  const price = useMemo(
    () =>
      new BigNumber(formatUnitPrice(item.unit_price, toCoinBalanceInfo.decimals))
        .times(fromDust(item.quantity, toCoinBalanceInfo.decimals))
        .toString(),
    [toCoinBalanceInfo.decimals, item.quantity, item.unit_price]
  );

  const handleBuyClick = () => {
    if (!account?.genRoochAddress().toHexAddress()) {
      return;
    }

    setConfirmData({
      price,
      quantity: Number(item.quantity),
    });
    setOpenDialog(true);
  };

  const handleConfirm = async () => {
    if (!account?.genRoochAddress().toHexAddress()) {
      return;
    }
    console.log(
      '🚀 ~ file: inscription-item-card.tsx:203 ~ item:',
      item,
      item.order_id,
      BigInt(item.order_id),
      Args.u64(BigInt(item.order_id))
    );
    setOpenDialog(false);

    const tx = new Transaction();
    tx.callFunction({
      target: `${market.orderBookAddress}::market_v2::buy`,
      args: [
        Args.objectId(market.tickInfo[tick].obj),
        Args.u64(BigInt(item.order_id)),
        Args.address(item.owner),
        Args.bool(true),
        Args.address(account.genRoochAddress().toStr()),
      ],
      typeArgs: [fromCoinBalanceInfo.coin_type, toCoinBalanceInfo.coin_type],
    });

    signAndExecuteTransaction(
      {
        transaction: tx,
      },
      {
        async onSuccess(data) {
          if (data.execution_info.status.type === 'executed') {
            toast.success('Buy Success');
            await onRefetchMarketData();
          } else {
            toast.error('Buy Failed');
          }
        },
        onError(error) {
          toast.error(String(error));
        },
      }
    );
  };

  const handleCancel = () => {
    setOpenDialog(false);
  };


  return (
    <Card
      key={item.order_id}
      sx={{
        '&:hover .add-cart-btn': {
          opacity: 1,
        },
        p: 1,
        cursor: selectMode ? 'pointer' : undefined,
        background: selectMode && selected ? grey[400] : undefined,
      }}
      onClick={() => {
        if (
          !selectMode ||
          !item.order_id ||
          item.owner === account?.genRoochAddress().toHexAddress()
        ) {
          return;
        }
        onSelectItem(item.order_id);
      }}
    >
      {selectMode && (
        <Stack direction="row" alignItems="center">
          <Checkbox
            size="medium"
            checked={selected}
            color="secondary"
            disabled={!item.order_id || item.owner === account?.genRoochAddress().toHexAddress()}
            icon={<Iconify icon="eva:radio-button-off-fill" />}
            checkedIcon={<Iconify icon="eva:checkmark-circle-2-fill" />}
            sx={{ p: 0.75 }}
          />
          {item.owner === account?.genRoochAddress().toHexAddress() && (
            <Chip size="small" disabled label="Owned" />
          )}
        </Stack>
      )}
      <InscriptionShopCard
        objectId={item.order_id}
        tick={tick}
        isVerified
        amount={item.quantity}
        price={price}
        unitPrice={formatUnitPrice(item.unit_price, toCoinBalanceInfo.decimals)}
        fromCoinBalanceInfo={fromCoinBalanceInfo}
        toCoinBalanceInfo={toCoinBalanceInfo}
        seller={item.owner}
        selectMode={selectMode}
        type="list"
      />
      {!selectMode && (
        <CardActions>
          <Stack
            direction="row"
            sx={{
              width: '100%',
            }}
            justifyContent="space-around"
            spacing={2}
          >
            {/* Buy */}
            {account?.genRoochAddress().toHexAddress() !== item.owner ? (
              <LoadingButton
                loading={isPending}
                disabled={
                  Boolean(!account?.genRoochAddress().toHexAddress()) ||
                  new BigNumber(accountBalance || 0).isLessThan(price)
                }
                variant="outlined"
                size="small"
                color="primary"
                fullWidth
                onClick={handleBuyClick}
              >
                {!account?.genRoochAddress().toHexAddress()
                  ? 'Please connect wallet'
                  : new BigNumber(accountBalance || 0).isLessThan(price)
                    ? 'Insufficient Balance'
                    : 'Buy'}
              </LoadingButton>
            ) : (
              <LoadingButton
                loading={isPending}
                variant="outlined"
                color="error"
                fullWidth
                size="small"
                onClick={() => {
                  // Cancel Order
                  console.log(
                    '🚀 ~ file: inscription-item-card.tsx:226 ~ fromCoinBalanceInfo:',
                    fromCoinBalanceInfo
                  );
                  const tx = new Transaction();
                  tx.callFunction({
                    target: `${market.orderBookAddress}::market_v2::cancel_order`,
                    args: [
                      Args.objectId(market.tickInfo[tick].obj),
                      Args.u64(BigInt(item.order_id)),
                    ],
                    typeArgs: [fromCoinBalanceInfo.coin_type, toCoinBalanceInfo.coin_type],
                  });
                  signAndExecuteTransaction(
                    {
                      transaction: tx,
                    },
                    {
                      async onSuccess(data) {
                        if (data.execution_info.status.type === 'executed') {
                          toast.success('Delist Success');
                          await onRefetchMarketData();
                        } else {
                          toast.error('Delist Failed');
                        }
                      },
                      onError(error) {
                        toast.error(String(error));
                      },
                    }
                  );
                }}
              >
                Delist
              </LoadingButton>
            )}
          </Stack>
        </CardActions>
      )}

      {/* Confirm Purchase */}
      <Dialog open={openDialog} onClose={handleCancel}>
        <DialogContent>
          <p>
            <strong>Balance Changes</strong>
          </p>
          <p style={{ color: 'green', margin: 0, textAlign: 'right' }}>
            + {confirmData.quantity} {tick.toUpperCase()}
          </p>
          <p style={{ color: 'red', margin: 0, textAlign: 'right' }}>
            - {new BigNumber(price).isNaN()
            ? '--'
            : formatNumber(fromDust(price, fromCoinBalanceInfo.decimals).toNumber())}{' '}
            {fromCoinBalanceInfo.symbol}
          </p>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCancel} color="primary" variant="outlined">
            Cancel
          </Button>
          <Button onClick={handleConfirm} color="primary" variant="contained">
            Confirm
          </Button>
        </DialogActions>
      </Dialog>
    </Card>
  );
}
