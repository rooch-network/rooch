import { AspectRatio } from '@/components/ui/aspect-ratio'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { useState } from 'react'

export const NftCard = () => {
  const nftData = [
    {
      id: 1,
      imageUrl:
        'https://i.seadn.io/s/raw/files/96f26dfaeb80982b4c48ef7b6d1a42a1.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 2,
      imageUrl:
        'https://i.seadn.io/s/raw/files/7700594825d9090b03f7134a9f96d9f0.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 3,
      imageUrl:
        'https://i.seadn.io/s/raw/files/d0f989ab16333bbf348fc74f0d4a6d8d.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 4,
      imageUrl:
        'https://i.seadn.io/s/raw/files/c8edb3d3eb5549a10f3cd2a919c7e6e6.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 5,
      imageUrl:
        'https://i.seadn.io/s/raw/files/96f26dfaeb80982b4c48ef7b6d1a42a1.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 6,
      imageUrl:
        'https://i.seadn.io/s/raw/files/7700594825d9090b03f7134a9f96d9f0.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 7,
      imageUrl:
        'https://i.seadn.io/s/raw/files/d0f989ab16333bbf348fc74f0d4a6d8d.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
    {
      id: 8,
      imageUrl:
        'https://i.seadn.io/s/raw/files/c8edb3d3eb5549a10f3cd2a919c7e6e6.png?auto=format&dpr=1&w=640',
      title: 'Rooch OG NFT',
      price: '6.988 ETH',
    },
  ]

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
        <div
          onClick={handleCloseModal}
          className="fixed inset-0 bg-black bg-opacity-50 flex justify-center items-center z-50"
        >
          <div className="bg-white rounded-lg flex flex-col items-start justify-center">
            <div className="p-4">
              <img
                src={selectedImage}
                alt="Selected NFT"
                className="w-[480px] h-auto md:w-[640px] rounded-lg"
              />
            </div>
          </div>
        </div>
      )}
    </>
  )
}
