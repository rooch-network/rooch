import type { AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

import { useState, useEffect } from 'react';
import {
  useRoochClient,
  useCurrentAddress,
} from '@roochnetwork/rooch-sdk-kit';

import { Card, Table, Stack, TableBody, Pagination } from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatCoin } from 'src/utils/format-number';

import { Scrollbar } from 'src/components/scrollbar';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import AllLiquidityRowItem from './add-liquidity-modal';
import LiquidityRowItem from './all-liquidity-row-item';
import { useAllLiquidity } from '../../hooks/use-all-liquidity';

import type { AllLiquidityItemType } from '../../hooks/use-all-liquidity';

const headerLabel = [
  { id: 'create_at', label: 'Create At' },
  { id: 'x', label: 'X' },
  { id: 'y', label: 'Y' },
  { id: 'action', label: 'Action', align: 'right' },
];

export default function AllLiquidityList() {
  const dex = useNetworkVariable('dex');
  const client = useRoochClient();
  const currentAddress = useCurrentAddress();
  const [tokenPairInfos, setTokenPairInfos] = useState<Map<string, { x: string; y: string }>>(
    new Map()
  );

  const { isPending, lpTokens, hasNext, index, paginate } = useAllLiquidity();

  useEffect(() => {
    if (!client || !currentAddress) {
      return;
    }
    const fetch = async () => {
      const infos = new Map();
      const promises = lpTokens.map(async (item) => {
        const [xResult, xCoinResult, yResult, yCoinResult] = await Promise.all([
          client.queryObjectStates({
            filter: {
              object_id: item.x.id,
            },
          }),
          client.getBalance({
            owner: currentAddress!.toStr(),
            coinType: item.x.type,
          }),
          client.queryObjectStates({
            filter: {
              object_id: item.y.id,
            },
          }),
          client.getBalance({
            owner: currentAddress!.toStr(),
            coinType: item.y.type,
          }),
        ]);

        const xBalance = (xResult.data[0].decoded_value!.value.balance as AnnotatedMoveStructView)
          .value.value as string;
        const yBalance = (yResult.data[0].decoded_value!.value.balance as AnnotatedMoveStructView)
          .value.value as string;

        infos.set(item.id, {
          x: formatCoin(Number(xBalance), xCoinResult.decimals, 2),
          y: formatCoin(Number(yBalance), yCoinResult.decimals, 2),
        });
      });

      await Promise.all(promises);

      setTokenPairInfos(infos);
    };

    fetch();
  }, [lpTokens, currentAddress, client]);

  console.log(hasNext, index);

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
      <Scrollbar sx={{ minHeight: 462 }}>
        <Table sx={{ minWidth: 720 }} size="medium">
          <TableHeadCustom headLabel={headerLabel} />

          <TableBody>
            {isPending ? (
              <TableSkeleton col={headerLabel.length} row={5} rowHeight="77px" />
            ) : (
              <>
                {lpTokens.map((row) => (
                  <LiquidityRowItem
                    key={row.id}
                    row={row}
                    balance={tokenPairInfos.get(row.id)}
                    onOpenViewModal={handleRemoveModal}
                  />
                ))}
                <TableNoData title="No Coins Found" notFound={lpTokens.length === 0} />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>
      {(hasNext || index > 1) && (
        <Stack className="mb-4 w-full mt-4" alignItems="flex-end">
          <Pagination
            count={hasNext ? index + 1 : index}
            page={index}
            onChange={(event: React.ChangeEvent<unknown>, value: number) => {
              paginate(value);
            }}
          />
        </Stack>
      )}
      {selectedRow && (
        <AllLiquidityRowItem
          open={openAddLiquidityModal}
          onClose={handleCloseRemoveModal}
          row={selectedRow}
          key={openAddLiquidityModal ? 'open' : 'closed'}
        />
      )}
    </Card>
  );
}
