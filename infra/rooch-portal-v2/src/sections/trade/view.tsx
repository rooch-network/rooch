'use client';

import type { BalanceInfoView, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

import { usePagination } from 'react-use-pagination';
import { Args, Serializer } from '@roochnetwork/rooch-sdk';
import { useMemo, useState, useEffect, useCallback } from 'react';
import { useRoochClient, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import Box from '@mui/material/Box';
import { LoadingButton } from '@mui/lab';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import { Tab, Card, Tabs, Alert, Stack, Button, Tooltip, Skeleton, useTheme } from '@mui/material';

import {
  type BidItem,
  type MarketItem,
  countMaxOccurrences,
} from 'src/hooks/trade/use-market-data';

import { fromDust } from 'src/utils/number';
import { shortCoinType } from 'src/utils/common';
import { fNumber } from 'src/utils/format-number';

import { secondary } from 'src/theme/core';

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

import { formatUnitPrice } from '../../utils/marketplace';
import { useNetworkVariable } from '../../hooks/use-networks';

export default function MarketplaceView({ params }: { params: { tick: string } }) {
  const { tick: marketplaceTick }: { tick: string } = params;

  const theme = useTheme();

  const market = useNetworkVariable('market');
  const client = useRoochClient();
  const account = useCurrentAddress();

  const [mergeMode] = useState(false);
  const [bidList, setBidList] = useState<BidItem[]>([]);
  const [loadingList, setLoadingList] = useState(false);
  const [listDialogOpen, setListDialogOpen] = useState(false);
  const [acceptBidItem, setAcceptBidItem] = useState<BidItem>();
  const [marketList, setMarketList] = useState<MarketItem[]>([]);
  const [mergeSelected, setMergeSelected] = useState<string[]>([]);
  const [currentTab, setCurrentTab] = useState<'list' | 'bid'>('list');
  const [acceptBidDialogOpen, setAcceptBidDialogOpen] = useState(false);
  const [floorPrice, setFloorPrice] = useState<string | undefined>();
  const [toCoinBalanceInfo, setToCoinBalanceInfo] = useState<BalanceInfoView>();
  const [fromCoinBalanceInfo, setFromCoinBalanceInfo] = useState<BalanceInfoView>();

  const renderList = useMemo(() => marketList, [marketList]);

  const tickLowerCase = useMemo(() => marketplaceTick.toLowerCase(), [marketplaceTick]);
  const tickUpperCase = useMemo(() => marketplaceTick.toUpperCase(), [marketplaceTick]);

  const onSelectMergeItem = useCallback(
    (inputValue: string) => {
      const newSelected = mergeSelected.includes(inputValue)
        ? mergeSelected.filter((value) => value !== inputValue)
        : [...mergeSelected, inputValue];

      setMergeSelected(newSelected);
    },
    [mergeSelected]
  );

  const { currentPage, totalPages, setPage, setNextPage, startIndex, endIndex } = usePagination({
    totalItems: renderList?.length || 0,
    initialPage: 0,
    initialPageSize: 50,
  });

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

  const [listPageParams, setListPageParams] = useState<{
    fromPrice: number;
    startOrderId: string;
    hasNextPage: boolean;
  }>({
    fromPrice: 0,
    startOrderId: '',
    hasNextPage: true,
  });

  const getListData = useCallback(async () => {
    if (!toCoinBalanceInfo || !fromCoinBalanceInfo) return;
    setLoadingList(true);
    try {
      let { fromPrice } = listPageParams;
      const { startOrderId } = listPageParams;
      console.log(
        '🚀 ~ file: view.tsx:123 ~ getListData ~ fromPrice, start:',
        fromPrice,
        startOrderId
      );
      const res = await client.executeViewFunction({
        target: `${market.orderBookAddress}::market_v2::query_order_info`,
        args: [
          Args.objectId(market.tickInfo[marketplaceTick].obj),
          Args.bool(false),
          Args.u64(fromPrice <= 0 ? 0n : BigInt(fromPrice)),
          Args.bool(fromPrice <= 0),
          Args.u64(Number(startOrderId) <= 0 ? 0n : BigInt(startOrderId)),
        ],
        typeArgs: [fromCoinBalanceInfo.coin_type, toCoinBalanceInfo.coin_type],
      });
      console.log(res);
      const decodedValue = res.return_values?.[0]?.decoded_value as AnnotatedMoveStructView;
      if ((decodedValue as any).length === 0) {
        // setMarketList([]);
        return;
      }
      const marketItemList = (decodedValue.value as unknown as any[]).map((i) => ({
        order_id: i[0],
        unit_price: i[1],
        quantity: i[2],
        owner: i[3],
        is_bid: i[4],
      })) as unknown as MarketItem[];
      setFloorPrice(marketItemList[0].unit_price);
      console.log('🚀 ~ file: view.tsx:140 ~ marketItemList ~ marketItemList:', marketItemList);
      setMarketList((prevData) => [...prevData, ...marketItemList]);
      const { fromPrice: countedFromPrice, start: countedStart } = countMaxOccurrences(
        marketItemList.map((item) => Number(item.unit_price))
      );
      console.log(
        '🚀 ~ file: view.tsx:159 ~ getListData ~ countedFromPrice:',
        countedFromPrice,
        countedStart
      );
      if (!(fromPrice === countedFromPrice && countedStart === 50)) {
        fromPrice = countedFromPrice;
      }
      setListPageParams(() => ({
        fromPrice,
        startOrderId: marketItemList[marketItemList.length - 1].order_id,
        hasNextPage: marketItemList.length === 50,
      }));
    } catch (error) {
      console.log('🚀 ~ file: view.tsx:231 ~ getListData ~ error:', error);
    } finally {
      setLoadingList(false);
    }
  }, [
    toCoinBalanceInfo,
    fromCoinBalanceInfo,
    listPageParams,
    client,
    market.orderBookAddress,
    market.tickInfo,
    marketplaceTick,
  ]);

  const getBidData = useCallback(async () => {
    if (!toCoinBalanceInfo || !fromCoinBalanceInfo) return;
    setLoadingList(true);
    try {
      const fromPrice = 0;
      const start = 0;
      const res = await client.executeViewFunction({
        target: `${market.orderBookAddress}::market_v2::query_order_info`,
        args: [
          Args.objectId(market.tickInfo[marketplaceTick].obj),
          Args.bool(true),
          Args.u64(fromPrice < 0 ? 0n : BigInt(fromPrice)),
          Args.bool(true),
          Args.u64(start < 0 ? 0n : BigInt(start)),
        ],
        typeArgs: [fromCoinBalanceInfo.coin_type, toCoinBalanceInfo.coin_type],
      });
      const decodedValue = res.return_values?.[0]?.decoded_value as AnnotatedMoveStructView;
      if ((decodedValue as any).length === 0) {
        setBidList([]);
        return;
      }
      const marketItemList = (decodedValue.value as unknown as any[]).map((i) => ({
        order_id: i[0],
        unit_price: i[1],
        quantity: i[2],
        owner: i[3],
        is_bid: i[4],
      })) as unknown as BidItem[];
      setBidList(marketItemList);
    } catch (error) {
      console.log('🚀 ~ file: view.tsx:260 ~ getBidData ~ error:', error);
    } finally {
      setLoadingList(false);
    }
  }, [client, fromCoinBalanceInfo, marketplaceTick, market, toCoinBalanceInfo]);

  const getMarketTradeInfo = useCallback(async () => {
    const res = await client.queryObjectStates({
      filter: {
        object_id: market.tickInfo[marketplaceTick].obj,
      },
    });
    const typeTag = Serializer.typeTagParseFromStr(res.data[0].object_type, true) as any;
    const fromCoin = typeTag.struct.typeParams[0].struct;
    const toCoin = typeTag.struct.typeParams[1].struct;
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
  }, [account, client, marketplaceTick, market]);

  useEffect(() => {
    getBidData();
  }, []);

  useEffect(() => {
    getListData();
  }, [toCoinBalanceInfo, fromCoinBalanceInfo]);

  useEffect(() => {
    getMarketTradeInfo();
  }, [getMarketTradeInfo]);

  const [createBidDialogOpen, setCreateBidDialogOpen] = useState(false);

  const sellList = useMemo(() => renderList, [renderList]);
  console.log('🚀 ~ file: view.tsx:275 ~ MarketplaceView ~ renderList:', renderList);

  const [showLimitTips, setShowLimitTips] = useState(false);

  const isWalletConnect = useMemo(
    () => Boolean(account?.genRoochAddress().toHexAddress()),
    [account]
  );

  return (
    <Container
      maxWidth="xl"
      sx={{
        position: 'relative',
        mb: 4,
      }}
    >
      {showLimitTips && account?.genRoochAddress().toHexAddress() && (
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
        </Stack>

        <Stack direction="row" alignItems="center">
          <LoadingButton
            loading={loadingList}
            variant="outlined"
            startIcon={<Iconify icon="solar:refresh-bold" width={24} />}
            onClick={async () => {
              setShowMyOrder(false);
              setLoadingOrder(false);
              await Promise.all([getListData(), getBidData(), getMarketTradeInfo()]);
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
                fNumber(
                  fromDust(
                    Number(toCoinBalanceInfo?.supply || 0),
                    toCoinBalanceInfo?.decimals || 0
                  ).toNumber()
                )
              ) : (
                <Skeleton variant="rounded" />
              )
            }
            icon="solar:pie-chart-2-bold-duotone"
            color={secondary.main}
          />
        </Stack>
      </Card>

      {!account?.genRoochAddress().toHexAddress() && (
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
        {loadingList && (sellList.length === 0 || !fromCoinBalanceInfo || !toCoinBalanceInfo)
          ? renderSkeleton
          : fromCoinBalanceInfo &&
            toCoinBalanceInfo && (
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
                              await Promise.all([
                                getListData(),
                                getBidData(),
                                getMarketTradeInfo(),
                              ]);
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
                          await Promise.all([getBidData()]);
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
            theme.palette.mode === 'light' ? 'rgba(244, 246, 248, 0.95)' : 'rgba(22, 28, 36, 0.95)',
        }}
      >
        <LoadingButton
          size="small"
          variant="contained"
          loading={loadingList}
          disabled={!listPageParams.hasNextPage}
          onClick={async () => {
            await getListData();
          }}
          sx={{
            ml: 1,
            height: '32px',
          }}
          endIcon={
            listPageParams.hasNextPage && <Iconify icon="icon-park-outline:right" width={16} />
          }
        >
          {!listPageParams.hasNextPage ? 'All Loaded' : 'Load More'}
        </LoadingButton>
      </Box>
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
          floorPrice={floorPrice ? formatUnitPrice(floorPrice, toCoinBalanceInfo.decimals) : ''}
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
          floorPrice={floorPrice ? formatUnitPrice(floorPrice, toCoinBalanceInfo.decimals) : ''}
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
