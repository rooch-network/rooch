import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { useMemo, useState } from 'react';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Card, Table, TableBody, CardHeader } from '@mui/material';

import { BitcoinAddressToRoochAddress } from 'src/utils/address';

import { Scrollbar } from 'src/components/scrollbar';
import TableSkeleton from 'src/components/skeleton/table-skeleton';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import AssetRowItem from './asset-row-item';
import CoinTransferModal from './coin-transfer-modal';

const headerLabel = [
  { id: 'coin', label: 'Coin' },
  { id: 'balance', label: 'Balance' },
  { id: 'action', label: 'Action', align: 'right' },
];

export default function AssetsTableCard({ address, dense }: { address: string; dense?: boolean }) {
  const currentAddress = useCurrentAddress();

  const {
    data: assetsList,
    isPending,
    refetch: refetchAssetsList,
  } = useRoochClientQuery(
    'getBalances',
    {
      owner: BitcoinAddressToRoochAddress(address).toHexAddress(),
    },
    { refetchInterval: 5000 }
  );
  console.log('🚀 ~ file: assets-table-card.tsx:31 ~ AssetsTableCard ~ assetsList:', assetsList);

  const isWalletOwner = useMemo(
    () => Boolean(currentAddress) && currentAddress?.toStr() === address,
    [currentAddress, address]
  );

  const filteredHeaderLabel = useMemo(
    () => (isWalletOwner ? headerLabel : headerLabel.slice(0, -1)),
    [isWalletOwner]
  );

  const [openTransferModal, setOpenTransferModal] = useState(false);
  const [selectedRow, setSelectedRow] = useState<BalanceInfoView>();

  const handleOpenTransferModal = (row: BalanceInfoView) => {
    setSelectedRow(row);
    setOpenTransferModal(true);
  };

  const handleCloseTransferModal = () => {
    setOpenTransferModal(false);
    setSelectedRow(undefined);
  };

  return (
    <Card className="mt-4">
      <CardHeader
        title="Coin"
        subheader={dense ? undefined : `Rooch network coin assets`}
        sx={{ mb: 3 }}
      />

      <Scrollbar sx={{ minHeight: dense ? undefined : 462 }}>
        <Table sx={{ minWidth: 720 }} size={dense ? 'small' : 'medium'}>
          <TableHeadCustom headLabel={filteredHeaderLabel} />

          <TableBody>
            {isPending ? (
              <TableSkeleton col={isWalletOwner ? 3 : 2} row={5} rowHeight="77px" />
            ) : (
              <>
                {assetsList?.data.map((row) => (
                  <AssetRowItem
                    key={row.coin_type}
                    row={row}
                    isWalletOwner={isWalletOwner}
                    onOpenTransferModal={handleOpenTransferModal}
                  />
                ))}
                <TableNoData title="No Coins Found" notFound={assetsList?.data.length === 0} />
              </>
            )}
          </TableBody>
        </Table>
      </Scrollbar>

      {selectedRow && (
        <CoinTransferModal
          open={openTransferModal}
          onClose={handleCloseTransferModal}
          selectedRow={selectedRow}
          refetch={refetchAssetsList}
          key={openTransferModal ? 'open' : 'closed'}
        />
      )}
    </Card>
  );
}
