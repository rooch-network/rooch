import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import type { MarketItem } from 'src/hooks/trade/use-market-data';

import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, UseSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { grey } from '@mui/material/colors';
import { Card, Chip, Stack, Checkbox, CardActions } from '@mui/material';

import { fromDust } from 'src/utils/number';

import { NETWORK, NETWORK_PACKAGE } from 'src/config/trade';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';

import { Iconify } from 'src/components/iconify';

import { toast } from '../snackbar';
import InscriptionShopCard from './inscription-shop-card';

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
  const { mutate: signAndExecuteTransaction, isPending } = UseSignAndExecuteTransaction();

  const price = useMemo(
    () =>
      new BigNumber(item.unit_price)
        .times(fromDust(item.quantity, toCoinBalanceInfo.decimals))
        .toString(),
    [toCoinBalanceInfo.decimals, item.quantity, item.unit_price]
  );

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
        // color: secondary['main'],
      }}
      onClick={() => {
        if (!selectMode || !item.order_id || item.owner === account?.genRoochAddress().toStr()) {
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
            disabled={!item.order_id || item.owner === account?.genRoochAddress().toStr()}
            icon={<Iconify icon="eva:radio-button-off-fill" />}
            checkedIcon={<Iconify icon="eva:checkmark-circle-2-fill" />}
            sx={{ p: 0.75 }}
          />
          {item.owner === account?.genRoochAddress().toStr() && (
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
        unitPrice={item.unit_price}
        // acc={item.acc}
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
            {account?.genRoochAddress().toStr() !== item.owner ? (
              <LoadingButton
                loading={isPending}
                disabled={
                  Boolean(!account?.genRoochAddress().toStr()) ||
                  new BigNumber(accountBalance || 0).isLessThan(price)
                }
                variant="outlined"
                size="small"
                color="primary"
                fullWidth
                onClick={() => {
                  if (!account?.genRoochAddress().toStr()) {
                    return;
                  }
                  const tx = new Transaction();
                  tx.callFunction({
                    target: `${TESTNET_ORDERBOOK_PACKAGE}::market_v2::buy`,
                    args: [
                      Args.objectId(NETWORK_PACKAGE.testnet.tickInfo[tick].MARKET_OBJECT_ID),
                      Args.u64(BigInt(item.order_id)),
                      Args.address(item.owner),
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
                        toast.success('Buy Success');
                        await onRefetchMarketData();
                      },
                      onError(error) {
                        toast.error(String(error));
                      },
                    }
                  );
                }}
              >
                {!account?.genRoochAddress().toStr()
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
                  const tx = new Transaction();
                  tx.callFunction({
                    target: `${TESTNET_ORDERBOOK_PACKAGE}::market::cancel_order`,
                    args: [
                      Args.objectId(NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_OBJECT_ID),
                      Args.u64(BigInt(item.order_id)),
                    ],
                    typeArgs: ['0x3::gas_coin::RGas', toCoinBalanceInfo.coin_type],
                  });
                  signAndExecuteTransaction(
                    {
                      transaction: tx,
                    },
                    {
                      async onSuccess(data) {
                        toast.success('Delist Success');
                        await onRefetchMarketData();
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
    </Card>
  );
}
