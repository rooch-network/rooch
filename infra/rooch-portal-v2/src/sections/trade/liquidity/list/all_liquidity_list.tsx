import type {
  IndexerStateIDView,
  AnnotatedMoveStructView,
} from '@roochnetwork/rooch-sdk';

import { useRef, useMemo, useState, useEffect } from 'react';
import {
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { Card , Table, Stack, TableBody, Pagination } from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { Scrollbar } from 'src/components/scrollbar';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import AllLiquidityRowItem from './add-liquidity-modal';
import LiquidityRowItem from './all-liquidity-row-item';

import type { AllLiquidityItemType } from './all-liquidity-row-item';

const headerLabel = [
  { id: 'create_at', label: 'Create At' },
  { id: 'x', label: 'X' },
  { id: 'y', label: 'Y' },
  { id: 'creator', label: 'Creator' },
  { id: 'action', label: 'Action', align: 'right' },
];

export default function AllLiquidityList() {
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();
  const [paginationModel, setPaginationModel] = useState({ index: 1, limit: 10 });
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateIDView | null }>({});

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.index - 1]?.toString(),
      limit: paginationModel.limit.toString(),
    }),
    [paginationModel]
  );

  const { data: tokenPairs, isPending } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type: `${dex.address}::swap::TokenPair`,
    },
    queryOption: {
      showDisplay: true,
    },
  });

  const resolvedTokenPairs = useMemo(() => {
    if (!tokenPairs) {
      return [];
    }

    const rowItme: AllLiquidityItemType[] = tokenPairs!.data.map((item) => {
      const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
      let xType = xView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
      xType = xType.replace('>>', '');
      const xName = xType.split('::');
      const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
      let yType = yView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
      yType = yType.replace('>>', '');
      const yName = yType.split('::');
      const lpView = item.decoded_value!.value.coin_info as AnnotatedMoveStructView;
      return {
        id: item.id,
        creator: item.decoded_value!.value.creator as string,
        createAt: Number(item.created_at),
        lpTokenId: lpView.value.id as string,
        x: {
          id: xView.value.id as string,
          symbol: xName[xName.length - 1].replace('>>', ''),
          type: xType,
        },
        y: {
          id: yView.value.id as string,
          symbol: yName[xName.length - 1].replace('>>', ''),
          type: yType,
        },
      };
    });

    return rowItme;
  }, [tokenPairs]);

  const paginate = (index: number): void => {
    if (index < 0) {
      return;
    }
    setPaginationModel({
      ...paginationModel,
      index,
    });
  };

  useEffect(() => {
    if (!tokenPairs) {
      return;
    }
    if (tokenPairs.has_next_page) {
      mapPageToNextCursor.current[paginationModel.index] = tokenPairs.next_cursor ?? null;
    }
  }, [paginationModel, tokenPairs]);

  // const {
  //   data: assetsList,
  //   isPending,
  //   refetch: refetchAssetsList,
  // } = useRoochClientQuery(
  //   'getBalances',
  //   {
  //     owner: currentAddress?.toStr() || '',
  //   },
  //   { refetchInterval: 5000 }
  // );

  const [openAddLiquidityModal, setOpenAddLiquidityModal] = useState(false);
  const [selectedRow, setSelectedRow] = useState<AllLiquidityItemType>();

  const handleRemoveModal = (row: AllLiquidityItemType) => {
    setSelectedRow(row);
    setOpenAddLiquidityModal(true);
  };

  const handleCloseRemoveModal = () => {
    setOpenAddLiquidityModal(false);
    setSelectedRow(undefined);
  };

  return (
    <Card className="mt-4">
      {/* <Box
        display="flex"
        justifyContent="space-between"
        alignItems="center"
        sx={{ ml: 2, mr: 1, height: 60 }}
      >
        <Box display="flex" width="100%" justifyContent="flex-end" alignItems="center">
          <Link href="./pool/add">
            <Button variant="outlined">Create Liquidity</Button>
          </Link>
        </Box>
      </Box> */}
      <Scrollbar sx={{ minHeight: 462 }}>
        <Table sx={{ minWidth: 720 }} size="medium">
          <TableHeadCustom headLabel={headerLabel} />

          <TableBody>
            {isPending ? (
              <TableSkeleton col={5} row={5} rowHeight="77px" />
            ) : (
              <>
                {resolvedTokenPairs.map((row) => (
                  <LiquidityRowItem key={row.id} row={row} onOpenViewModal={handleRemoveModal} />
                ))}
                <TableNoData title="No Coins Found" notFound={resolvedTokenPairs.length === 0} />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
      <Stack className="mb-4 w-full mt-4" alignItems="flex-end">
        <Pagination
          count={tokenPairs?.has_next_page ? paginationModel.index + 1 : paginationModel.index}
          page={paginationModel.index}
          onChange={(event: React.ChangeEvent<unknown>, value: number) => {
            paginate(value);
          }}
        />
      </Stack>
      {selectedRow && (
        <AllLiquidityRowItem
          open={openAddLiquidityModal}
          onClose={handleCloseRemoveModal}
          row={selectedRow}
          // refetch={refetchAssetsList}
          key={openAddLiquidityModal ? 'open' : 'closed'}
        />
      )}
    </Card>
  );
}
