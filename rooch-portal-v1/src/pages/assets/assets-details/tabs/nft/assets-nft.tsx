import { useEffect, useState } from 'react'

import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { AspectRatio } from '@/components/ui/aspect-ratio'

import { ArrowLeft, Copy } from 'lucide-react'

import { formatAddress } from '@/utils/format'
import {
  useCurrentSession,
  useRoochClient,
  useRoochClientQuery,
  useTransferObject,
} from '@roochnetwork/rooch-sdk-kit'
import { ROOCH_OPERATING_ADDRESS } from '@/common/constant.ts'
import { NoData } from '@/components/no-data.tsx'

export const AssetsNft = () => {
  const sessionKey = useCurrentSession()
  const [modalOpen, setModalOpen] = useState(false)
  const [selectedNFTId, setSelectedNFTId] = useState('')
  // const [curNFT, setCurNFT] = useState<ObjectStateView>()
  const [images, setImages] = useState<Map<string, string>>(new Map())
  const [toAddress, setToAddress] = useState('')
  const [transferLoading, setTransferLoading] = useState(false)

  const client = useRoochClient()

  const { mutateAsync: transferObject } = useTransferObject()

  // TODO: How do I get all the nft
  // TODO: 1, data/image loading, 2, pagination
  const { data: nfts, refetch: reFetchNFTS } = useRoochClientQuery('queryGlobalStates', {
    filter: {
      object_type_with_owner: {
        owner: sessionKey?.getAddress() || '',
        object_type: `${ROOCH_OPERATING_ADDRESS}::nft::NFT`,
      },
    },
    cursor: null,
    limit: 10,
    descending_order: true,
  })

  // fetch collection info
  useEffect(() => {
    const fetchCollectionInfo = async () => {
      if (!nfts || nfts.data.length === 0) {
        return
      }
      const collectionInfo = await Promise.all(
        nfts.data
          .map((item) => ({
            key: item.object_id,
            collection: item.value.value.collection,
          }))
          .map(async (obj) => {
            const result = await client.getStates({ accessPath: `/object/${obj.collection}` })
            console.log(result)
            return {
              key: obj.key,
              image: (result[0].decoded_value as any).value.value.value.url + '?auto=format&dpr=1&w=640',
            }
          }),
      )

      const map = collectionInfo.reduce((map, item) => {
        map.set(item.key, item.image)
        return map
      }, new Map<string, string>())

      setImages(map)
    }


    fetchCollectionInfo().then()
  }, [client, nfts])

  // ** modal 打开时，禁止父组件 scroll
  useEffect(() => {
    if (modalOpen) {
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = ''
    }

    return () => {
      document.body.style.overflow = ''
    }
  }, [modalOpen])

  // ** ESC 关闭 modal
  useEffect(() => {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.keyCode === 27) {
        setModalOpen(false)
      }
    }

    window.addEventListener('keydown', handleEsc)

    return () => {
      window.removeEventListener('keydown', handleEsc)
    }
  }, [])

  const handleImageClick = (nftId: string) => {
    setSelectedNFTId(nftId)
    setModalOpen(true)
  }

  const handleClose = () => {
    setModalOpen(false)
  }

  const handleCloseModal = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    if (event.target === event.currentTarget) {
      handleClose()
    }
  }

  const handleTransferObject = async () => {
    const nft = nfts?.data.find((item) => item.object_id === selectedNFTId)

    if (!nft || toAddress === '') {
      return
    }

    setTransferLoading(true)

    await transferObject({
      account: sessionKey!,
      toAddress: toAddress,
      objId: nft.object_id,
      objType: nft.object_type,
    })

    handleClose()
    setTransferLoading(false)
    reFetchNFTS()
  }

  return !nfts || nfts.data.length === 0 ? (
    <NoData />
  ) : (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 w-full place-items-center">
      {nfts?.data.map((nft) => (
        <Card
          key={nft.object_id}
          className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          onClick={() => handleImageClick(nft.object_id)}
        >
          <CardContent className="p-0">
            <AspectRatio ratio={1 / 1} className="flex items-center justify-center overflow-hidden">
              <img
                src={images.get(nft.object_id)}
                alt="NFT Image"
                className="rounded-md object-cover hover:scale-110 transition-all ease-in-out duration-300"
              />
            </AspectRatio>
          </CardContent>
          <CardHeader className="px-4 md:px-6">
            <CardTitle>{nft.value.value.name as string}</CardTitle>
            {/*<CardDescription>{nft.price}</CardDescription>*/}
          </CardHeader>
          <CardFooter className="px-4 md:px-6">
            <Button variant="default" size="default" className="w-full">
              Transfer
            </Button>
          </CardFooter>
        </Card>
      ))}

      {modalOpen && (
        <div className="flex items-center justify-center font-mono">
          <div
            className="fixed inset-0 bg-opacity-70 dark:bg-opacity-75 flex justify-center items-center z-50 bg-black"
            onClick={handleCloseModal}
          >
            <div className="bg-background dark:bg-zinc-900 rounded-none md:rounded-lg flex flex-col items-start justify-center p-6 w-full h-full md:w-auto md:h-auto overflow-auto">
              {/* Back */}
              <div className="mb-4">
                <Button
                  variant="secondary"
                  size="sm"
                  className="h-8 w-14 rounded-2xl bg-accent dark:bg-zinc-800 dark:hover:bg-zinc-700/65"
                  onClick={handleClose}
                >
                  <ArrowLeft className="w-5 h-5 text-muted-foreground dark:text-gray-200" />
                </Button>
              </div>

              {/* Content */}
              <div className="flex flex-col md:flex-row h-full items-center justify-start md:items-start md:justify-start gap-6 md:gap-12 md:mr-6">
                {/* NFT Image */}
                <div>
                  <img
                    src={images.get(selectedNFTId)}
                    alt="Selected NFT"
                    className="w-full md:max-w-[420px] h-auto rounded-lg shadow-md"
                  />
                </div>

                {/* Transfer Description */}
                <div className="flex flex-col items-start justify-start gap-3 w-full md:w-[320px]">
                  {/* From Address */}
                  <div className="cursor-pointer">
                    <span className="text-base font-normal text-gray-800 dark:text-gray-100 flex items-center justify-start gap-2 transition-all">
                      <p>{formatAddress(sessionKey?.getAddress())}</p>
                      <Copy className="w-4 h-4 text-muted-foreground" />
                    </span>
                  </div>

                  {/* Send */}
                  <span className="text-muted-foreground dark:text-zinc-500 font-normal">Send</span>

                  {/* NFT Name */}
                  <span className="text-gray-800 dark:text-gray-50 text-3xl font-normal tracking-wide">
                    Rooch OG NFT
                  </span>

                  {/* To */}
                  <span className="text-muted-foreground dark:text-zinc-500 font-normal">To</span>

                  {/* To Address (Input) */}
                  <Input
                    className="h-12 rounded-2xl bg-gray-50 text-gray-800"
                    placeholder="bc1pr6mdxnc348lua02c32ad4uyyaw3kavjz4c8jzkh5ffvuq4ryvxhsf879j5"
                    value={toAddress}
                    onChange={(event) => {
                      setToAddress(event.target.value)
                    }}
                  />

                  {/* CTA */}
                  <Button
                    variant="default"
                    size="default"
                    onClick={handleTransferObject}
                    disabled={transferLoading}
                    className="w-full mt-6 md:mt-24 font-sans"
                  >
                    {transferLoading ? 'Transferring' : 'Transfer'}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
