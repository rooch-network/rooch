import { useEffect, useState } from 'react'

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { AspectRatio } from '@/components/ui/aspect-ratio'

import { ArrowLeft, Copy } from 'lucide-react'

import { nftData } from '@/common/constant'
import { formatAddress } from '../../../../../../utils/format'

export const NftCard = () => {
  const [modalOpen, setModalOpen] = useState(false)
  const [selectedImage, setSelectedImage] = useState('')

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

  const handleImageClick = (imageUrl: string) => {
    setSelectedImage(imageUrl)
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

  return (
    <>
      {nftData.map((nft) => (
        <Card
          key={nft.id}
          className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          onClick={() => handleImageClick(nft.imageUrl)}
        >
          <CardContent className="p-0">
            <AspectRatio ratio={1 / 1} className="flex items-center justify-center overflow-hidden">
              <img
                src={nft.imageUrl}
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
                    src={selectedImage}
                    alt="Selected NFT"
                    className="w-full md:max-w-[420px] h-auto rounded-lg shadow-md"
                  />
                </div>

                {/* Transfer Description */}
                <div className="flex flex-col items-start justify-start gap-3 w-full md:w-[320px]">
                  {/* From Address */}
                  <div className="cursor-pointer">
                    <span className="text-base font-normal text-gray-800 dark:text-gray-100 flex items-center justify-start gap-2 transition-all">
                      <p>
                        {formatAddress(
                          'bc1pk33n2t5zulq7nz3k8rq55dywjt89szfukypjua5zmuhfg40338wsv9ss7q',
                        )}
                      </p>
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
                  />

                  {/* CTA */}
                  <Button
                    variant="default"
                    size="default"
                    className="w-full mt-6 md:mt-24 font-sans"
                  >
                    Transfer
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
