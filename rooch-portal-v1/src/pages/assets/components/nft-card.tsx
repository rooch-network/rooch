import { useState } from 'react'

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { AspectRatio } from '@/components/ui/aspect-ratio'

import { nftData } from '@/common/constant'
import { ArrowLeft, Copy } from 'lucide-react'
import { formatAddress } from '../../../utils/format'
import { Input } from '@/components/ui/input'

console.log(nftData)

export const NftCard = () => {
  const [modalOpen, setModalOpen] = useState(false)
  const [selectedImage, setSelectedImage] = useState('')

  const handleImageClick = (imageUrl: string) => {
    setSelectedImage(imageUrl)
    setModalOpen(true)
  }

  const handleCloseModal = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    if (event.target === event.currentTarget) {
      setModalOpen(false)
    }
  }

  return (
    <>
      {nftData.map((nft) => (
        <Card
          key={nft.id}
          className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
        >
          <CardContent className="p-0">
            <AspectRatio
              ratio={1 / 1}
              className="flex items-center justify-center overflow-hidden cursor-pointer"
            >
              <img
                src={nft.imageUrl}
                onClick={() => handleImageClick(nft.imageUrl)}
                alt="NFT Image"
                className="rounded-md object-cover hover:scale-110 transition-all ease-in-out duration-300"
              />
            </AspectRatio>
          </CardContent>
          <CardHeader className="px-4 md:px-6">
            <CardTitle>{nft.title}</CardTitle>
            <CardDescription>{nft.price}</CardDescription>
          </CardHeader>
          <CardFooter className="px-4 md:px-6">
            <Button variant="default" size="default" className="w-full font-bold">
              Transfer
            </Button>
          </CardFooter>
        </Card>
      ))}

      {modalOpen && (
        <div className="flex items-center justify-center font-mono">
          <div
            className="fixed inset-0 bg-opacity-50 flex justify-center items-center z-50"
            onClick={handleCloseModal}
          >
            <div className="bg-zinc-900 rounded-lg flex flex-col items-start justify-center p-6">
              {/* Back */}
              <div className="mb-4">
                <Button variant="secondary" size="sm" className="h-8 w-14 rounded-2xl">
                  <ArrowLeft className="w-5 h-5 text-gray-200" />
                </Button>
              </div>

              {/* Content */}
              <div className="flex items-start justify-start gap-12 mr-6">
                {/* NFT Image */}
                <div>
                  <img
                    src={selectedImage}
                    alt="Selected NFT"
                    className="w-full md:max-w-[420px] h-auto rounded-lg"
                  />
                </div>

                {/* Transfer Description */}
                <div className="flex flex-col items-start justify-start gap-3 w-[320px]">
                  {/* From Address */}
                  <div>
                    <span className="text-base font-normal text-gray-100 flex items-center justify-start gap-2">
                      <p>
                        {formatAddress(
                          'bc1pk33n2t5zulq7nz3k8rq55dywjt89szfukypjua5zmuhfg40338wsv9ss7q',
                        )}
                      </p>
                      <Copy className="w-4 h-4 text-muted-foreground" />
                    </span>
                  </div>

                  {/* Send */}
                  <span className="text-zinc-500 font-normal">Send</span>

                  {/* NFT Name */}
                  <span className="text-gray-50 text-3xl font-normal tracking-wide">
                    Rooch OG NFT
                  </span>

                  {/* To */}
                  <span className="text-zinc-500 font-normal">To</span>

                  {/* To Address (Input) */}
                  <Input
                    className="h-12 rounded-2xl bg-gray-50 text-gray-800"
                    placeholder="bc1pr6mdxnc348lua02c32ad4uyyaw3kavjz4c8jzkh5ffvuq4ryvxhsf879j5"
                  />

                  {/* CTA */}
                  <button
                    type="button"
                    className="text-white bg-gradient-to-br from-green-400 to-blue-600 hover:bg-gradient-to-bl focus:ring-4 focus:outline-none focus:ring-green-200 dark:focus:ring-green-800 font-medium rounded-lg text-sm px-5 py-2.5 text-center me-2 mt-24 duration-300 ease-in-out w-full h-11"
                  >
                    Transfer
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
