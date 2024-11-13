'use client';

import type { BidItem, MarketItem } from 'src/hooks/trade/use-market-data';
import type { BalanceInfoView, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

import BigNumber from 'bignumber.js';
import { useCountDown } from 'ahooks';
import { usePagination } from 'react-use-pagination';
import { useQueryClient } from '@tanstack/react-query';
import { Args, Serializer } from '@roochnetwork/rooch-sdk';
import { useMemo, useState, useEffect, useCallback } from 'react';
import { useRoochClient, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import Box from '@mui/material/Box';
import { LoadingButton } from '@mui/lab';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import {
  Tab,
  Card,
  Tabs,
  Alert,
  Stack,
  Button,
  Tooltip,
  Skeleton,
  useTheme,
  Pagination,
} from '@mui/material';

import { fromDust } from 'src/utils/number';
import { fNumber } from 'src/utils/format-number';
import { sleep, shortCoinType } from 'src/utils/common';

import { SUI_DECIMALS } from 'src/config/trade';
import { grey, secondary } from 'src/theme/core';
import { TESTNET_ORDERBOOK_PACKAGE } from 'src/config/constant';

import { Iconify } from 'src/components/iconify';
import ListDialog from 'src/components/market/list-dialog';
import { EmptyContent } from 'src/components/empty-content';
import TradeInfoItem from 'src/components/market/trade-info-item';
import CreateBidDialog from 'src/components/market/create-bid-dialog';
import AcceptBidDialog from 'src/components/market/accept-bid-dialog';
import InscriptionItemCard from 'src/components/market/inscription-item-card';
import { renderSkeleton } from 'src/components/skeleton/product-item-skeleton-list';
import { ProductItemSkeleton } from 'src/components/skeleton/product-item-skeleton';
import InscriptionItemBidCard from 'src/components/market/inscription-item-bid-card';

export default function MarketplaceView({ params }: { params: { tick: string } }) {
  const { tick: marketplaceTick }: { tick: string } = params;

  const tickLowerCase = useMemo(() => marketplaceTick.toLowerCase(), [marketplaceTick]);

  const tickUpperCase = useMemo(() => marketplaceTick.toUpperCase(), [marketplaceTick]);

  const theme = useTheme();

  const [currentTab, setCurrentTab] = useState<'list' | 'bid'>('list');

  const account = useCurrentAddress();

  const client = useRoochClient();

  const [mergeSelected, setMergeSelected] = useState<string[]>([]);

  const [fromCoinBalanceInfo, setFromCoinBalanceInfo] = useState<BalanceInfoView>();
  console.log(
    '🚀 ~ file: view.tsx:72 ~ MarketplaceView ~ fromCoinBalanceInfo:',
    fromCoinBalanceInfo
  );

  const [toCoinBalanceInfo, setToCoinBalanceInfo] = useState<BalanceInfoView>();

  const onSelectMergeItem = useCallback(
    (inputValue: string) => {
      const newSelected = mergeSelected.includes(inputValue)
        ? mergeSelected.filter((value) => value !== inputValue)
        : [...mergeSelected, inputValue];

      setMergeSelected(newSelected);
    },
    [mergeSelected]
  );

  const [mergeMode, setMergeMode] = useState(false);

  const queryClient = useQueryClient();

  const [listPageParams, setListPageParams] = useState<{
    fromPrice: number;
    start: number;
    hasNextPage: boolean;
  }>({
    fromPrice: 0,
    start: 0,
    hasNextPage: true,
  });

  const [marketList, setMarketList] = useState<MarketItem[]>([]);
  console.log('🚀 ~ file: view.tsx:101 ~ MarketplaceView ~ marketList:', marketList);
  const [bidList, setBidList] = useState<BidItem[]>([]);

  const [loadingList, setLoadingList] = useState(false);

  const [floorPrice, setFloorPrice] = useState<number | undefined>();

  const renderList = useMemo(() => marketList, [marketList]);

  const { currentPage, totalPages, setPage, setNextPage, startIndex, endIndex } = usePagination({
    totalItems: renderList?.length || 0,
    initialPage: 0,
    initialPageSize: 50,
  });

  const [targetDate, setTargetDate] = useState<number | undefined>(Date.now() + 10 * 1000);

  const [countdown] = useCountDown({
    targetDate,
    onEnd: async () => {
      // await fetchMarketList(true);
      setTargetDate(Date.now() + 10 * 1000);
    },
  });

  const [acceptBidItem, setAcceptBidItem] = useState<BidItem>();
  const [acceptBidDialogOpen, setAcceptBidDialogOpen] = useState(false);

  const [listDialogOpen, setListDialogOpen] = useState(false);

  const openAcceptBidDialog = (item: BidItem) => {
    setAcceptBidItem(item);
    setAcceptBidDialogOpen(true);
  };

  const closeAcceptBidDialog = () => {
    setAcceptBidDialogOpen(false);
    setAcceptBidItem(undefined);
  };

  const [showMyOrder, setShowMyOrder] = useState(false);

  const [loadingOrder, setLoadingOrder] = useState(false);

  const [fromCoinStruct, setFromCoinStruct] = useState<{
    address: string;
    module: string;
    name: string;
  }>();

  const [toCoinStruct, setToCoinStruct] = useState<{
    address: string;
    module: string;
    name: string;
  }>();

  const getListData = useCallback(async () => {
    setLoadingList(true);
    try {
      const fromPrice = 0;
      const start = 0;
      const res = await client.executeViewFunction({
        target: `${TESTNET_ORDERBOOK_PACKAGE}::market_v2::query_order_info`,
        args: [
          Args.objectId('0x156d9a5bfa4329f999115b5febde94eed4a37cde10637ad8eed1ba91e89e0bb7'),
          Args.bool(false),
          Args.u64(fromPrice < 0 ? 0n : BigInt(fromPrice)),
          Args.bool(true),
          Args.u64(start < 0 ? 0n : BigInt(start)),
        ],
        typeArgs: [
          '0x3::gas_coin::RGas',
          '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::fixed_supply_coin::FSC',
        ],
      });
      const decodedValue = res.return_values?.[0]?.decoded_value as AnnotatedMoveStructView[];
      const marketItemList = decodedValue.map((i) => i.value) as unknown as MarketItem[];
      setMarketList(marketItemList);
    } catch (error) {
      console.log('🚀 ~ file: view.tsx:231 ~ getListData ~ error:', error);
    } finally {
      setLoadingList(false);
    }
  }, [client]);

  const getBidData = useCallback(async () => {
    setLoadingList(true);
    try {
      const fromPrice = 0;
      const start = 0;
      const res = await client.executeViewFunction({
        target: `${TESTNET_ORDERBOOK_PACKAGE}::market_v2::query_order_info`,
        args: [
          Args.objectId('0x156d9a5bfa4329f999115b5febde94eed4a37cde10637ad8eed1ba91e89e0bb7'),
          Args.bool(true),
          Args.u64(fromPrice < 0 ? 0n : BigInt(fromPrice)),
          Args.bool(true),
          Args.u64(start < 0 ? 0n : BigInt(start)),
        ],
        typeArgs: [
          '0x3::gas_coin::RGas',
          '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::fixed_supply_coin::FSC',
        ],
      });
      const decodedValue = res.return_values?.[0]?.decoded_value as AnnotatedMoveStructView[];
      const marketItemList = decodedValue.map((i) => i.value) as unknown as BidItem[];
      setBidList(marketItemList);
    } catch (error) {
      console.log('🚀 ~ file: view.tsx:260 ~ getBidData ~ error:', error);
    } finally {
      setLoadingList(false);
    }
  }, [client]);

  const getMarketTradeInfo = useCallback(async () => {
    const res = await client.queryObjectStates({
      filter: {
        object_id: '0x4386917e0942a6fb30048405e3326be618bd0661b2fe240a112d6b59afb94cbb',
      },
    });
    const typeTag = Serializer.typeTagParseFromStr(res.data[0].object_type, true) as any;
    const fromCoin = typeTag.struct.typeParams[0].struct;
    const toCoin = typeTag.struct.typeParams[1].struct;
    setFromCoinStruct({
      address: fromCoin.address,
      module: fromCoin.module,
      name: fromCoin.name,
    });
    setToCoinStruct({
      address: toCoin.address,
      module: toCoin.module,
      name: toCoin.name,
    });
    const [fromCoinBalanceInfo, toCoinBalanceInfo] = await Promise.all([
      client.queryObjectStates({
        filter: {
          object_type: `0x3::coin::CoinInfo<${fromCoin.address}::${fromCoin.module}::${fromCoin.name}>`,
        },
        queryOption: {
          decode: true,
        },
      }),
      client.queryObjectStates({
        filter: {
          object_type: `0x3::coin::CoinInfo<${toCoin.address}::${toCoin.module}::${toCoin.name}>`,
        },
        queryOption: {
          decode: true,
        },
      }),
    ]);
    const tempFromCoinBalanceInfo = {
      ...fromCoinBalanceInfo.data[0].decoded_value?.value,
      balance: '0',
    };
    const tempToCoinBalanceInfo = {
      ...toCoinBalanceInfo.data[0].decoded_value?.value,
      balance: '0',
    };
    if (account) {
      const [fromBalanceInfo, toBalanceInfo] = await Promise.all([
        client.getBalance({
          owner: account.genRoochAddress().toStr(),
          coinType: `${fromCoin.address}::${fromCoin.module}::${fromCoin.name}`,
        }),
        client.getBalance({
          owner: account.genRoochAddress().toStr(),
          coinType: `${toCoin.address}::${toCoin.module}::${toCoin.name}`,
        }),
      ]);
      tempFromCoinBalanceInfo.balance = fromBalanceInfo.balance;
      tempToCoinBalanceInfo.balance = toBalanceInfo.balance;
    }

    setFromCoinBalanceInfo(tempFromCoinBalanceInfo as BalanceInfoView);
    setToCoinBalanceInfo(tempToCoinBalanceInfo as BalanceInfoView);
  }, [account, client]);

  useEffect(() => {
    getBidData();
  }, [getBidData]);

  useEffect(() => {
    getListData();
  }, [getListData]);

  useEffect(() => {
    getMarketTradeInfo();
  }, [getMarketTradeInfo]);

  // useEffect(() => {
  //   getListData();
  //   getBidData();
  //   getMarketTradeInfo();
  // }, [getListData, getBidData, getUserBalance, getMarketTradeInfo]);

  const [createBidDialogOpen, setCreateBidDialogOpen] = useState(false);

  const sellList = useMemo(
    () => (totalPages <= 1 ? renderList : renderList.slice(startIndex, endIndex + 1)),
    [renderList, totalPages, startIndex, endIndex]
  );

  const [showLimitTips, setShowLimitTips] = useState(false);

  const selectedTotalAmount = useMemo(() => {
    let totalAmount = new BigNumber(0);
    mergeSelected.forEach((id) => {
      const item = sellList.find((listItem) => listItem.order_id === id);
      if (item) {
        totalAmount = totalAmount.plus(item.quantity);
      }
    });
    return totalAmount.toNumber();
  }, [mergeSelected, sellList]);

  const selectedTotalPrice = useMemo(() => {
    let totalPrice = new BigNumber(0);
    mergeSelected.forEach((id) => {
      const item = sellList.find((listItem) => listItem.quantity === id);
      if (item) {
        totalPrice = totalPrice.plus(new BigNumber(item.unit_price).times(item.quantity));
      }
    });
    return totalPrice.toNumber();
  }, [mergeSelected, sellList]);

  const isWalletConnect = useMemo(() => Boolean(account?.genRoochAddress().toStr()), [account]);

  return (
    <Container
      maxWidth="xl"
      sx={{
        position: 'relative',
        mb: 4,
      }}
    >
      {showLimitTips && account?.genRoochAddress().toStr() && (
        <Alert
          severity="success"
          variant="outlined"
          sx={{
            mb: 2,
          }}
          onClose={() => {
            setShowLimitTips(false);
          }}
        >
          The marketplace is now open, with more advanced features under development and coming
          soon!
        </Alert>
      )}
      {showMyOrder && (
        <Alert
          severity="warning"
          variant="filled"
          sx={{
            mb: 2,
          }}
        >
          {`Fetching the user's listing records may take some time. Please be patient and wait.`}
        </Alert>
      )}
      {/* <Alert
        severity="success"
        variant="outlined"
        sx={{
          mb: 2,
        }}
      >
        Bid is now live. Welcome to give it a try!
      </Alert> */}
      <Stack direction="row" justifyContent="space-between" alignItems="center" flexWrap="wrap">
        <Stack direction="row" alignItems="center" spacing={4} flexWrap="wrap">
          <Typography variant="h4"> Marketplace | {tickUpperCase} </Typography>
          {isWalletConnect && (
            <>
              <Button
                variant="contained"
                color="secondary"
                onClick={() => {
                  setListDialogOpen(true);
                }}
              >
                List
              </Button>
              <Button
                variant="contained"
                color="secondary"
                onClick={() => {
                  setCreateBidDialogOpen(true);
                }}
              >
                Bid
              </Button>
            </>
          )}
          {/* {account?.genRoochAddress().toStr() && (
            <FormControlLabel
              control={
                <Switch
                  size="medium"
                  color="secondary"
                  checked={mergeMode}
                  onChange={(e, value) => {
                    setMergeMode(value);
                    if (value) {
                      setTargetDate(undefined);
                    } else {
                      setTargetDate(Date.now() + 10 * 1000);
                    }
                  }}
                />
              }
              label={
                <Typography
                  sx={{
                    fontWeight: 600,
                  }}
                >
                  Batch Mode
                </Typography>
              }
            />
          )} */}
        </Stack>

        <Stack direction="row" alignItems="center">
          {/* {isWalletConnect && (
            <FormControlLabel
              control={
                <Switch
                  size="medium"
                  color="secondary"
                  checked={showMyOrder}
                  disabled={loadingOrder}
                  onChange={(e, value) => {
                    if (value) {
                      setTargetDate(undefined);
                    } else {
                      setTargetDate(Date.now() + 10 * 1000);
                    }
                    setShowMyOrder(value);
                  }}
                />
              }
              label={
                <Typography
                  sx={{
                    fontWeight: 600,
                    display: 'flex',
                    alignItems: 'center',
                  }}
                >
                  Show My Order{' '}
                  {loadingOrder && (
                    <CircularProgress
                      size={18}
                      sx={{
                        ml: 1,
                      }}
                    />
                  )}
                </Typography>
              }
            />
          )} */}
          <LoadingButton
            loading={loadingList}
            variant="outlined"
            startIcon={<Iconify icon="solar:refresh-bold" width={24} />}
            onClick={async () => {
              setShowMyOrder(false);
              setLoadingOrder(false);
              await Promise.all([getListData(), getBidData(), getMarketTradeInfo()]);
              setTargetDate(Date.now() + 10 * 1000);
            }}
            sx={{
              ml: 2,
            }}
            suppressHydrationWarning
          >
            Refresh
          </LoadingButton>
        </Stack>
      </Stack>

      {/* Trade Info */}
      <Card
        sx={{
          mt: 2,
          mb: 2,
          boxShadow: 'none',
        }}
        variant="outlined"
      >
        <Stack
          direction="row"
          // divider={<Divider orientation="vertical" sx={{ borderStyle: 'dashed' }} />}
          justifyContent="start"
          gap={2}
          display="flex"
          className="justify-between"
          // gridTemplateColumns={{
          //   xs: 'repeat(1, 1fr)',
          //   sm: 'repeat(1, 1fr)',
          //   md: 'repeat(3, 1fr)',
          //   lg: 'repeat(5, 1fr)',
          // }}
          sx={{ py: 2, pl: 2, pr: 2 }}
        >
          <TradeInfoItem
            title="Coin"
            value={
              <Typography
                sx={{
                  fontWeight: 600,
                }}
              >
                {tickUpperCase}
              </Typography>
            }
            icon="solar:sale-bold"
            color={secondary.main}
          />

          <TradeInfoItem
            title="Coin Name"
            value={toCoinBalanceInfo ? toCoinBalanceInfo?.name : <Skeleton variant="rounded" />}
            icon="solar:tag-bold"
            color={secondary.main}
          />

          <TradeInfoItem
            title="Coin Type"
            value={
              toCoinBalanceInfo ? (
                <Tooltip title={toCoinBalanceInfo?.coin_type}>
                  <span>{shortCoinType(toCoinBalanceInfo?.coin_type)}</span>
                </Tooltip>
              ) : (
                <Skeleton variant="rounded" />
              )
            }
            icon="solar:chart-2-bold-duotone"
            color={secondary.main}
          />

          <TradeInfoItem
            title="Total Supply"
            value={
              toCoinBalanceInfo ? (
                fNumber(toCoinBalanceInfo?.supply)
              ) : (
                <Skeleton variant="rounded" />
              )
            }
            icon="solar:pie-chart-2-bold-duotone"
            color={secondary.main}
          />
        </Stack>
      </Card>

      {!account?.genRoochAddress().toStr() && (
        <Alert
          variant="outlined"
          severity="info"
          sx={{
            mt: 4,
            mb: 2,
          }}
        >
          Please connect wallet to trade.
        </Alert>
      )}

      {mergeMode && (
        <Stack
          direction="row"
          alignItems="center"
          spacing={2}
          sx={{
            pt: 2,
            pb: 2,
            position: 'sticky',
            top: '60px',
            zIndex: 1,
            background:
              theme.palette.mode === 'light'
                ? 'rgba(244, 246, 248, 0.95)'
                : 'rgba(22, 28, 36, 0.95)',
          }}
          flexWrap="wrap"
        >
          <Button
            variant="outlined"
            color="info"
            onClick={() => {
              if (mergeSelected.length > 0) {
                setMergeSelected([]);
              } else {
                setMergeSelected(
                  sellList
                    .filter((item) => item.owner !== account?.genRoochAddress().toStr())
                    .map((item) => item.order_id)
                );
              }
            }}
          >
            Select All (Current Page)
          </Button>
          {/* <LoadingButton
            loading={isPending}
            variant="contained"
            color="success"
            disabled={
              mergeSelected.length <= 1 ||
              new BigNumber(accountBalance?.totalBalance || 0).isLessThan(selectedTotalPrice)
            }
            onClick={() => {
              if (!account?.genRoochAddress().toStr()) {
                return;
              }
              const txb = makeBatchBuyTxb(mergeSelected, account.address, sellList, tickLowerCase);

              txb.setSender(account.address);

              signAndExecuteTransactionBlock(
                {
                  transactionBlock: txb,
                  options: {
                    showObjectChanges: true,
                  },
                },
                {
                  async onSuccess(data) {
                    setMergeSelected([]);
                    toast.success('Buy Success');
                    await refetchMarketData();
                  },
                  onError(error) {
                    toast.error(String(error));
                  },
                }
              );
            }}
          >
            {new BigNumber(accountBalance?.totalBalance || 0).isLessThan(selectedTotalPrice) ? (
              'Insufficient Balance'
            ) : (
              <>Buy {mergeSelected.length > 1 ? mergeSelected.length : null} Inscription</>
            )}
          </LoadingButton> */}
          <Stack
            sx={{
              ml: {
                xs: undefined,
                sm: 'auto',
              },
              fontSize: '1.25rem',
            }}
          >
            <Stack direction="row">
              Total Price:{'  '}
              <span
                style={{
                  fontWeight: 600,
                  marginLeft: '12px',
                  fontSize: '1.3rem',
                  color: secondary.light,
                }}
              >
                {fromDust(selectedTotalPrice, SUI_DECIMALS).toFixed(5)} SUI
              </span>
            </Stack>
            <Stack
              direction="row"
              alignItems="center"
              sx={{
                fontSize: '0.875rem',
                color: secondary.light,
              }}
            >
              Total ${tickUpperCase}:{'  '}
              <span
                style={{
                  fontWeight: 400,
                  marginLeft: '12px',
                  fontSize: '1rem',
                  color: secondary.light,
                }}
              >
                {fNumber(selectedTotalAmount)}
              </span>
            </Stack>
            <Stack
              direction="row"
              alignItems="center"
              sx={{
                fontSize: '0.875rem',
                color: grey[600],
              }}
            >
              Avg. Price:{'  '}
              <span
                style={{
                  fontWeight: 400,
                  marginLeft: '12px',
                  fontSize: '1rem',
                  color: secondary.light,
                  marginRight: '4px',
                }}
              >
                {selectedTotalAmount === 0
                  ? '--'
                  : new BigNumber(fromDust(selectedTotalPrice, SUI_DECIMALS))
                      .div(selectedTotalAmount)
                      .toFixed(6)}
              </span>
              SUI/{tickUpperCase}
            </Stack>
          </Stack>
        </Stack>
      )}

      {/* {targetDate && (
        <Stack direction="row" alignItems="center" spacing={1}>
          <PuffLoader speedMultiplier={0.875} color={warning.light} loading size={24} />
          <Typography
            sx={{
              fontSize: '0.875rem',
              color: grey[600],
            }}
          >
            {loadingList ? (
              'Refreshing...'
            ) : (
              <span>Refresh after {Math.round(countdown / 1000)} second(s)</span>
            )}
          </Typography>
        </Stack>
      )} */}

      <Tabs
        value={currentTab}
        onChange={(e, value) => {
          setCurrentTab(value);
        }}
      >
        <Tab value="list" label="List" />
        <Tab value="bid" label="Bid" />
      </Tabs>

      {!loadingList && (currentTab === 'list' ? sellList.length === 0 : bidList.length === 0) && (
        <Box
          sx={{
            mt: 4,
            height: '20vh',
          }}
        >
          <EmptyContent
            title="No Record"
            sx={{
              py: 4,
            }}
          />
        </Box>
      )}

      <Box
        gap={3}
        display="grid"
        gridTemplateColumns={{
          xs: 'repeat(2, 1fr)',
          sm: 'repeat(3, 1fr)',
          md: 'repeat(4, 1fr)',
          lg: 'repeat(4, 1fr)',
        }}
        sx={{
          mt: 2,
        }}
      >
        {loadingList ||
        !sellList ||
        sellList.length === 0 ||
        !fromCoinBalanceInfo ||
        !toCoinBalanceInfo ? (
          renderSkeleton
        ) : (
          <>
            {currentTab === 'list' ? (
              <>
                {sellList.map(
                  (item) =>
                    item.order_id && (
                      <InscriptionItemCard
                        fromCoinBalanceInfo={fromCoinBalanceInfo}
                        toCoinBalanceInfo={toCoinBalanceInfo}
                        key={item.order_id}
                        item={item}
                        tick={tickLowerCase}
                        accountBalance={fromCoinBalanceInfo.balance}
                        selectMode={mergeMode}
                        selected={mergeSelected.includes(item.order_id)}
                        onSelectItem={onSelectMergeItem}
                        onRefetchMarketData={async () => {
                          await Promise.all([getListData(), getBidData(), getMarketTradeInfo()]);
                        }}
                      />
                    )
                )}
              </>
            ) : (
              <>
                {bidList?.map((item) => (
                  <InscriptionItemBidCard
                    fromCoinBalanceInfo={fromCoinBalanceInfo}
                    toCoinBalanceInfo={toCoinBalanceInfo}
                    item={item}
                    tick={tickLowerCase}
                    onRefetchMarketData={async () => {
                      setCurrentTab('bid');
                      await Promise.all([]);
                    }}
                    onAcceptBid={(item) => {
                      openAcceptBidDialog(item);
                    }}
                  />
                ))}
              </>
            )}
            {loadingOrder && <ProductItemSkeleton />}
          </>
        )}
      </Box>

      {currentTab === 'list' && !showMyOrder && sellList.length === 50 && (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            position: 'sticky',
            bottom: 0,
            mt: 2,
            p: 2,
            background:
              theme.palette.mode === 'light'
                ? 'rgba(244, 246, 248, 0.95)'
                : 'rgba(22, 28, 36, 0.95)',
          }}
        >
          <Pagination
            shape="rounded"
            page={currentPage + 1}
            count={totalPages}
            siblingCount={2}
            hideNextButton
            onChange={(e, value) => {
              setTargetDate(undefined);
              setPage(value - 1);
            }}
            variant="text"
          />
          <LoadingButton
            size="small"
            variant="contained"
            loading={loadingList}
            onClick={async () => {
              // await fetchMarketList();
              await sleep(10);
              setTargetDate(undefined);
              setNextPage();
            }}
            sx={{
              ml: 1,
              height: '32px',
            }}
            endIcon={<Iconify icon="icon-park-outline:right" width={16} />}
          >
            Load More
          </LoadingButton>
        </Box>
      )}
      {acceptBidItem && fromCoinBalanceInfo && toCoinBalanceInfo && (
        <AcceptBidDialog
          open={acceptBidDialogOpen}
          acceptBidItem={acceptBidItem}
          tick={tickLowerCase}
          fromCoinBalanceInfo={fromCoinBalanceInfo}
          toCoinBalanceInfo={toCoinBalanceInfo}
          tokenBalance={toCoinBalanceInfo.balance}
          refreshBidList={async () => {
            setCurrentTab('bid');
            await Promise.all([getListData(), getBidData(), getMarketTradeInfo()]);
            // await Promise.all([refetchBidList(), refetchAddressOwnedInscription()]);
          }}
          close={closeAcceptBidDialog}
        />
      )}
      {fromCoinBalanceInfo && toCoinBalanceInfo && createBidDialogOpen && (
        <CreateBidDialog
          tick={tickLowerCase}
          floorPrice={floorPrice ?? 0}
          open={createBidDialogOpen}
          fromCoinBalanceInfo={fromCoinBalanceInfo}
          toCoinBalanceInfo={toCoinBalanceInfo}
          refreshBidList={async () => {
            setCurrentTab('bid');
            await getBidData();
          }}
          close={() => {
            setCreateBidDialogOpen(false);
          }}
        />
      )}
      {fromCoinBalanceInfo && toCoinBalanceInfo && listDialogOpen && (
        <ListDialog
          listDialogOpen={listDialogOpen}
          close={() => {
            setListDialogOpen(false);
          }}
          floorPrice="1"
          tick={tickLowerCase}
          fromCoinBalanceInfo={fromCoinBalanceInfo}
          toCoinBalanceInfo={toCoinBalanceInfo}
          refreshList={async () => {
            await getListData();
          }}
        />
      )}
    </Container>
  );
}
