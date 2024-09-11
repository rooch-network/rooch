import type { IndexerStateIDView, IndexerObjectStateView } from '@roochnetwork/rooch-sdk';

import { useRef, useMemo, useState } from 'react';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Button, Skeleton, CardHeader, CardContent } from '@mui/material';

import { EmptyContent } from 'src/components/empty-content/empty-content';

import { FMNFT } from 'src/sections/mint/constant';

import NFTTransferModal from './nft-transfer-modal';

export default function NFTList({ address }: { address: string }) {
  const currentAddress = useCurrentAddress();
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 });
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateIDView | null }>({});

  const [openTransferModal, setOpenTransferModal] = useState(false);
  const [selectedNFT, setSelectedNFT] = useState<IndexerObjectStateView>();

  const isWalletOwner = useMemo(
    () => Boolean(currentAddress) && currentAddress?.toStr() === address,
    [currentAddress, address]
  );

  const handleOpenTransferModal = (row: IndexerObjectStateView) => {
    setSelectedNFT(row);
    setOpenTransferModal(true);
  };

  const handleCloseTransferModal = () => {
    setOpenTransferModal(false);
    setSelectedNFT(undefined);
  };

  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) return;

    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    });
  };

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1] || null,
      pageSize: paginationModel.pageSize.toString(),
    }),
    [paginationModel]
  );

  const {
    data: nftList,
    refetch: refetchNFTList,
    isPending: isNFTListPending,
  } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type_with_owner: {
        owner: address,
        object_type: FMNFT.objType,
      },
    },
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
    queryOption: {
      showDisplay: true,
      descending: true,
    },
  });

  return (
    <Card>
      <CardHeader title="NFT" sx={{ mb: 1 }} />
      <CardContent
        className="!pt-2"
        component={Box}
        gap={3}
        display="grid"
        gridTemplateColumns={
          nftList?.data.length === 0
            ? undefined
            : {
                xs: 'repeat(1, 1fr)',
                sm: 'repeat(2, 1fr)',
                md: 'repeat(3, 1fr)',
                lg: 'repeat(4, 1fr)',
              }
        }
      >
        {isNFTListPending ? (
          Array.from({ length: 4 }).map((i,index) => <Skeleton key={index} height={256} />)
        ) : nftList?.data.length === 0 ? (
          <EmptyContent title="No NFT Found" sx={{ py: 3 }} />
        ) : (
          nftList?.data.map((i) => (
            <Card key={i.id} elevation={0} className="!bg-gray-100 !shadow-none">
              <CardHeader
                title={i.display_fields?.fields.name as string}
                titleTypographyProps={{
                  sx: {
                    fontSize: '1rem !important',
                  },
                }}
              />
              <CardContent className="!pt-2 rounded">
                <Box
                  component="img"
                  className="aspect-square rounded-xl"
                  src={`data:image/svg+xml;base64,${btoa(i.display_fields?.fields.image_url as string)}`}
                />
                {isWalletOwner && (
                  <Button
                    variant="outlined"
                    className="!mt-4"
                    fullWidth
                    onClick={() => {
                      handleOpenTransferModal(i);
                    }}
                  >
                    Transfer
                  </Button>
                )}
              </CardContent>
            </Card>
          ))
        )}
      </CardContent>

      {selectedNFT && (
        <NFTTransferModal
          open={openTransferModal}
          onClose={handleCloseTransferModal}
          selectedNFT={selectedNFT}
          refetch={refetchNFTList}
          key={openTransferModal ? 'open' : 'closed'}
        />
      )}
    </Card>
  );
}
