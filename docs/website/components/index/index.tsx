import Image from 'next/image'
import { useEffect, useState } from 'react'
import '@fontsource/kanit/400.css'
import '@fontsource/kanit/500.css'
import '@fontsource/kanit/700.css'
import '@fontsource/kanit/900.css'

const ECOSYSTEM_IMAGE_URL = {
  btc: [
    'https://lombard.finance',
    'https://lorenzo-protocol.xyz/',
    'https://www.bedrock.technology/',
    'https://solv.finance',
    'https://mainnet.pumpbtc.xyz/',
    'https://x.com/FBTC_brc20',
    'https://pstake.finance',
  ],
  infra: [
    'https://portaltobitcoin.com/',
    'https://babylonchain.io',
    'https://www.gonative.cc/',
    'https://nubit.org',
    'https://www.apro.com/',
    'https://availproject.org',
    'https://meson.fi',
    'https://free.tech',
    'https://usher.so',
    'https://ankr.com',
  ],
  move: [
    'https://aptosfoundation.org/',
    'https://pontem.network',
    'https://movementlabs.xyz',
    'https://initia.xyz',
    'https://www.cetus.zone/',
    'https://www.echo-protocol.xyz/',
  ],
}

const BACKED_BY_URL = [
  'https://x.com/Sky9Capital',
  'https://x.com/gate_ventures',
  'https://x.com/Globalcoinrsrch',
  'https://x.com/AnkerOfficial',
  'https://x.com/Cointelegraph',
  'https://x.com/babylonlabs_io',
  'https://x.com/AvailProject',
  'https://x.com/Aptos',
]

const Index = () => {
  const [signboardStates, setSignboardStates] = useState({
    board1: false,
    board2: false,
    boardM: false,
    board3: false,
    board4: false,
  })

  const handleSignboardClick = (boardId: keyof typeof signboardStates) => {
    setSignboardStates((prev) => ({
      ...prev,
      [boardId]: true,
    }))
  }

  const [isMoveVmHovered, setIsMoveVmHovered] = useState(false)

  const [currentStart, setCurrentStart] = useState(1)
  const [translateX, setTranslateX] = useState(0)

  const [btcTranslateX, setBtcTranslateX] = useState(0)
  const [moveTranslateX, setMoveTranslateX] = useState(0)
  const [btcStart, setBtcStart] = useState(1)
  const [moveStart, setMoveStart] = useState(1)

  const BACKED_BY_IMAGE_COUNT = 8

  const INFRA_IMAGE_COUNT = 10
  const BTC_IMAGE_COUNT = 7
  const MOVE_IMAGE_COUNT = 6

  const getImages = (start: number, total: number) => {
    const result = []
    let current = start
    for (let i = 0; i < 8; i++) {
      result.push(current)
      current = current === total ? 1 : current + 1
    }
    return result
  }

  const ITEM_WIDTH = 191
  const GAP_WIDTH = 32
  const SLIDE_WIDTH = ITEM_WIDTH + GAP_WIDTH

  useEffect(() => {
    const intervals = [
      setInterval(() => {
        setTranslateX((prev) => prev - SLIDE_WIDTH)
        if (translateX <= -SLIDE_WIDTH * 3) {
          setCurrentStart((prev) => (prev === 5 ? 1 : prev + 1))
          setTranslateX(0)
        }
      }, 3000),

      setInterval(() => {
        setBtcTranslateX((prev) => prev - SLIDE_WIDTH)
        if (btcTranslateX <= -SLIDE_WIDTH * 3) {
          setBtcStart((prev) => (prev === 5 ? 1 : prev + 1))
          setBtcTranslateX(0)
        }
      }, 3000),

      setInterval(() => {
        setMoveTranslateX((prev) => prev - SLIDE_WIDTH)
        if (translateX <= -SLIDE_WIDTH * 3) {
          setMoveStart((prev) => (prev === 5 ? 1 : prev + 1))
          setMoveTranslateX(0)
        }
      }, 3000),
    ]

    return () => intervals.forEach(clearInterval)
  }, [translateX, btcTranslateX, moveTranslateX])

  const [carVisibility, setCarVisibility] = useState({
    left: false,
    middle: false,
    right: false,
  })

  const [carMoving, setCarMoving] = useState({
    left: false,
    middle: false,
    right: false,
  })
  const [isAnimating, setIsAnimating] = useState(false)

  const handleCarAnimation = (position: 'left' | 'middle' | 'right') => {
    if (isAnimating) return

    setIsAnimating(true)

    setCarVisibility((prev) => ({
      ...prev,
      [position]: true,
    }))

    setTimeout(() => {
      setCarMoving((prev) => ({
        ...prev,
        [position]: true,
      }))

      setTimeout(() => {
        setCarVisibility((prev) => ({
          ...prev,
          [position]: false,
        }))
        setCarMoving((prev) => ({
          ...prev,
          [position]: false,
        }))
        setIsAnimating(false)
      }, 2000)
    }, 100)
  }

  const [isDarkMode, setIsDarkMode] = useState(false)

  useEffect(() => {
    const isDark = document.documentElement.classList.contains('dark')
    setIsDarkMode(isDark)

    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.attributeName === 'class') {
          const isDark = document.documentElement.classList.contains('dark')
          setIsDarkMode(isDark)
        }
      })
    })

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    })

    return () => observer.disconnect()
  }, [])

  const getImagePath = (category: string, num: number) => {
    const darkSuffix = isDarkMode ? '-dark' : ''
    return `/home/${category}${darkSuffix}/${num}.svg`
  }

  const getBackedByImagePath = (num: number) => {
    const darkSuffix = isDarkMode ? 'dark' : 'light'
    return `/home/backed-by/${darkSuffix}/${num}.svg`
  }

  const getImageUrl = (category: string, num: number) => {
    return ECOSYSTEM_IMAGE_URL[category][num]
  }

  const getBackedByUrl = (num: number) => {
    return BACKED_BY_URL[num - 1]
  }

  return (
    <>
      <div className="antialiased overflow-x-hidden">
        {/* HERO */}
        <div className="flex flex-col items-center justify-center md:justify-center h-full px-4 sm:px-6 md:px-8 lg:px-20 dark:border-b dark:border-b-zinc-800 overflow-x-hidden overflow-y-hidden">
          <div
            style={{
              overflowX: 'hidden',
              backgroundImage: 'url(./home/background.svg)',
              backgroundSize: 'cover',
              opacity: '0.3',
              backgroundPosition: 'center',
              backgroundRepeat: 'no-repeat',
              width: '100%',
              height: '100vh',
              position: 'fixed',
              top: 0,
              left: 0,
              zIndex: -1,
            }}
          ></div>
          <div className="flex flex-col items-center justify-center w-full font-[Han] z-10 relative top-[3rem]">
            <div className="text-5xl md:text-5xl font-bold text-center text-black dark:text-[#EEEBEB]">
              Where Bitcoin Becomes More Than Money
            </div>
            <div className="mt-6 text-2xl text-center text-black dark:text-[#EAEAEA] max-w-3xl font-[Kanit]">
              Rooch Network is the MoveVM-based Bitcoin Layer 2, enabling Bitcoin programmable,
              scalable, and interoperable.
            </div>
          </div>
          <div className="flex flex-col items-center justify-center w-full relative top-[2rem] md:top-[0.2rem]">
            {/* signboard list */}
            <div className="text-center mx-auto flex items-end justify-center relative w-full">
              <div className="flex items-end justify-center w-full relative">
                <div
                  className="relative w-[28%] left-[2%] flex-shrink-0 z-10 cursor-pointer"
                  onClick={() => handleSignboardClick('board1')}
                >
                  <Image
                    src="/home/signboard-1-0.svg"
                    className={`relative w-[100%]`}
                    alt="signboard-1"
                    width={420.06}
                    height={616}
                    style={{ objectFit: 'contain' }}
                  />
                  <Image
                    src="/home/signboard-1-1.svg"
                    className={`absolute top-0 left-0 w-[100%] transition-opacity duration-300 ${
                      signboardStates.board1 ? 'opacity-100' : 'opacity-0'
                    }`}
                    alt="signboard-1-hover"
                    width={420.06}
                    height={616}
                    style={{ objectFit: 'contain' }}
                  />
                </div>
                <div
                  className="relative w-[15%] -left-[3%] flex-shrink-0 cursor-pointer"
                  onClick={() => handleSignboardClick('board2')}
                >
                  <Image
                    src="/home/signboard-2-0.svg"
                    className="relative w-[100%]"
                    alt="signboard-2"
                    width={206.56}
                    height={335.5}
                    style={{ objectFit: 'contain' }}
                  />
                  <Image
                    src="/home/signboard-2-1.svg"
                    className={`absolute top-0 left-0 w-[100%] transition-opacity duration-300 ${
                      signboardStates.board2 ? 'opacity-100' : 'opacity-0'
                    }`}
                    alt="signboard-2-hover"
                    width={206.56}
                    height={335.5}
                    style={{ objectFit: 'contain' }}
                  />
                </div>
                <div
                  className="relative w-[26%] flex-shrink-0 cursor-pointer"
                  onClick={() => handleSignboardClick('boardM')}
                >
                  <Image
                    src="/home/signboard-m-0.svg"
                    className="relative w-[100%]"
                    alt="signboard-m"
                    width={386}
                    height={393}
                    style={{ objectFit: 'contain' }}
                  />
                  <Image
                    src="/home/signboard-m-1.svg"
                    className={`absolute top-0 left-0 z-10 w-[100%] transition-opacity duration-300 ${
                      signboardStates.boardM ? 'opacity-100' : 'opacity-0'
                    }`}
                    alt="signboard-m-hover"
                    width={386}
                    height={393}
                    style={{ objectFit: 'contain' }}
                  />
                </div>
                <div
                  className="relative w-[20%] left-[1%] flex-shrink-0 z-10 cursor-pointer"
                  onClick={() => handleSignboardClick('board3')}
                >
                  <Image
                    src="/home/signboard-3-0.svg"
                    className="relative w-[100%]"
                    alt="signboard-3"
                    width={336.2}
                    height={376.5}
                    style={{ objectFit: 'contain' }}
                  />
                  <Image
                    src="/home/signboard-3-1.svg"
                    className={`absolute left-0 top-0 z-10 w-[100%] transition-opacity duration-300 ${
                      signboardStates.board3 ? 'opacity-100' : 'opacity-0'
                    }`}
                    alt="signboard-3-hover"
                    width={336.2}
                    height={376.5}
                    style={{ objectFit: 'contain' }}
                  />
                </div>
                <div
                  className="relative w-[21%] flex-shrink-0 cursor-pointer -left-[5rem]"
                  onClick={() => handleSignboardClick('board4')}
                >
                  <Image
                    src="/home/signboard-4-0.svg"
                    className="relative w-[100%]"
                    alt="signboard-4"
                    width={332.28}
                    height={617}
                    style={{ objectFit: 'contain' }}
                  />
                  <Image
                    src="/home/signboard-4-1.svg"
                    className={`absolute top-0 left-0 z-10 w-[100%] transition-opacity duration-300 ${
                      signboardStates.board4 ? 'opacity-100' : 'opacity-0'
                    }`}
                    alt="signboard-4-hover"
                    width={332.28}
                    height={617}
                    style={{ objectFit: 'contain' }}
                  />
                </div>
              </div>
            </div>
            <div>
              <style jsx>{`
                @keyframes scrollRight {
                  from {
                    background-position: 0 0;
                  }
                  to {
                    background-position: 100% 0;
                  }
                }
              `}</style>
              <div
                className="relative z-10"
                style={{
                  top: '-12px',
                  background: 'url(./home/cation.svg)',
                  animation: 'scrollRight 20s linear infinite',
                  width: '100vw',
                  height: '64px',
                  backgroundRepeat: 'repeat-x',
                }}
              ></div>
            </div>
          </div>
        </div>

        {/* FEATURES */}
        <div className="py-16 md:py-10 md:pt-0 px-4 sm:px-6 md:px-8 lg:px-20 dark:bg-inherit flex flex-col md:flex-col items-center justify-between gap-12 md:gap-8 dark:border-b dark:border-b-zinc-800">
          <div className="flex flex-col items-center justify-center w-full font-['Han']">
            <div className="mt-14 text-5xl md:text-5xl font-bold text-center text-black dark:text-[#EEEBEB]">
              MoveVM
            </div>
            <div className="mt-6 text-2xl text-center text-black dark:text-[#EAEAEA] max-w-3xl font-[Kanit]">
              The best choice of VM for Bitcoin DApps
            </div>
          </div>
          <div
            className="flex flex-wrap justify-center items-center w-full md:w-auto relative cursor-pointer"
            onMouseEnter={() => setIsMoveVmHovered(true)}
            onMouseLeave={() => setIsMoveVmHovered(false)}
          >
            <div className="relative w-full">
              <img
                src="/home/move-vm-light.svg"
                alt="features logo"
                className="w-full h-auto object-cover transition-opacity duration-300"
                style={{ opacity: isMoveVmHovered ? 0 : 1 }}
              />
              <img
                src="/home/move-vm-dark.svg"
                alt="features logo"
                className="w-full h-auto object-cover absolute inset-0 transition-opacity duration-300"
                style={{ opacity: isMoveVmHovered ? 1 : 0 }}
              />
            </div>
          </div>
          <div className="mt-6">
            <style jsx>{`
              @keyframes scrollRight {
                from {
                  background-position: 0 0;
                }
                to {
                  background-position: 100% 0;
                }
              }
            `}</style>
            <div
              className="relative z-10"
              style={{
                top: '-12px',
                background: 'url(./home/cation.svg)',
                animation: 'scrollRight 20s linear infinite',
                width: '100vw',
                height: '64px',
                backgroundRepeat: 'repeat-x',
              }}
            ></div>
          </div>
        </div>

        {/* FEATURES */}
        <div className="py-12 px-8 lg:px-20 dark:bg-inherit flex flex-col items-center justify-between gap-12 md:gap-8 dark:border-b dark:border-b-zinc-800">
          <div className="flex flex-col items-center justify-center w-full font-['Han']">
            <div className="mt-14 text-5xl md:text-5xl font-bold text-center text-black dark:text-[#EEEBEB]">
              BTC Staking
            </div>
            <div className="mt-6 text-2xl text-center text-black dark:text-[#EAEAEA] max-w-3xl font-[Kanit]">
              Generate yield for users in a non-custodial manner,
              <br />
              compatible with Babylon protocol
            </div>
          </div>
          <div className="flex flex-wrap justify-center items-center w-full md:w-auto relative cursor-pointer -mt-[7vw]">
            <div className="relative w-screen">
              <img src="/home/road-sign.svg" alt="road-sign" className="w-screen h-auto" />
            </div>
            <div className="relative w-screen mt-[120px]">
              {/* left car */}
              <img
                src="/home/car-l.svg"
                className={`absolute bottom-0 left-[10%] w-[25%] transform -translate-x-1/2 pointer-events-none transition-all duration-2000 ease-in-out
                  ${carMoving.left ? 'scale-[0.05] translate-y-[-15vw] translate-x-[6vw] opacity-0' : ''}`}
                style={{ display: carVisibility.left ? 'block' : 'none' }}
              />
              {/* mid card */}
              <img
                src="/home/car-m.svg"
                className={`absolute bottom-0 left-1/2 w-[25%] transform -translate-x-1/2 pointer-events-none transition-all duration-2000 ease-in-out
                  ${carMoving.middle ? 'scale-[0.05] translate-y-[-15vw] opacity-0' : ''}`}
                style={{ display: carVisibility.middle ? 'block' : 'none' }}
              />
              {/* right car */}
              <img
                src="/home/car-r.svg"
                className={`absolute bottom-0 left-[90%] w-[25%] transform -translate-x-1/2 pointer-events-none transition-all duration-2000 ease-in-out
                  ${carMoving.right ? 'scale-[0.05] translate-y-[-15vw] translate-x-[-34vw] opacity-0' : ''}`}
                style={{ display: carVisibility.right ? 'block' : 'none' }}
              />
              <img src="/home/ground.svg" alt="road-sign" className="w-screen h-auto" />
              <button
                className="absolute inset-0 w-1/3 bottom-0 left-[18%] transform -translate-x-1/2 h-full bg-transparent cursor-pointer"
                onClick={() => handleCarAnimation('left')}
                disabled={isAnimating}
              />
              <button
                className="absolute inset-0 w-1/3 bottom-0 left-1/2 transform -translate-x-1/2 h-full bg-transparent cursor-pointer"
                onClick={() => handleCarAnimation('middle')}
                disabled={isAnimating}
              />
              <button
                className="absolute inset-0 w-1/3 bottom-0 left-[82%] transform -translate-x-1/2 h-full bg-transparent cursor-pointer"
                onClick={() => handleCarAnimation('right')}
                disabled={isAnimating}
              />
            </div>
          </div>
          <div className="relative -mt-[3vw]">
            <style jsx>{`
              @keyframes scrollRight {
                from {
                  background-position: 0 0;
                }
                to {
                  background-position: 100% 0;
                }
              }
            `}</style>
            <div
              className="relative z-10"
              style={{
                background: 'url(./home/cation.svg)',
                animation: 'scrollRight 20s linear infinite',
                width: '100vw',
                height: '64px',
                backgroundRepeat: 'repeat-x',
              }}
            ></div>
          </div>
        </div>

        {/* ECOSYSTEM PARTNER */}
        <div className="py-10 px-4 sm:px-6 md:px-8 lg:px-20 dark:bg-inherit flex flex-col md:flex-col items-center justify-center gap-6 md:gap-8 dark:border-b dark:border-b-zinc-800">
          <div className="my-7 text-5xl md:text-5xl font-['Han'] font-bold text-center text-black dark:text-[#EEEBEB]">
            Backed by
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-16 place-items-center place-content-center w-full mb-7">
            {Array.from({ length: BACKED_BY_IMAGE_COUNT }, (_, index) => (
              <div key={`backed-by-${index}`} className="flex items-center justify-center">
                <img
                  src={getBackedByImagePath(index + 1)}
                  className="flex-shrink-0 cursor-pointer"
                  onClick={() => {
                    window.open(getBackedByUrl(index + 1), '_blank')
                  }}
                />
              </div>
            ))}
          </div>
          <div className="mt-14 mb-7 text-5xl md:text-5xl font-['Han'] font-bold text-center text-black dark:text-[#EEEBEB]">
            BTC Staking
          </div>
          <div className="w-full h-full flex items-center justify-center">
            <div className="flex flex-col gap-y-12 font-['Han'] text-center">
              <div className="text-3xl w-[318px] h-[150px] font-medium rounded-[28px] border-[#81B39F] border-[3px] flex items-center justify-center">
                Infra
              </div>
              <div className="text-3xl w-[318px] h-[150px] font-medium rounded-[28px] border-[#81B39F] border-[3px] flex items-center justify-center">
                BTC <br /> Liquidity
              </div>
              <div className="text-3xl w-[318px] h-[150px] font-medium rounded-[28px] border-[#81B39F] border-[3px] flex items-center justify-center">
                Move <br /> Ecosystem
              </div>
            </div>
            <div className="flex flex-col overflow-hidden w-[892px] ml-12">
              {/* Infra */}
              <div
                className="flex gap-8 items-center transition-transform duration-500 ease-in-out"
                style={{ transform: `translateX(${translateX}px)` }}
              >
                {getImages(currentStart, INFRA_IMAGE_COUNT).map((num, index) => (
                  <img
                    key={`${num}-${index}`}
                    src={getImagePath('infra', num)}
                    className="w-[191px] flex-shrink-0 cursor-pointer"
                    onClick={() => {
                      window.open(getImageUrl('infra', num - 1), '_blank')
                    }}
                  />
                ))}
              </div>

              {/* BTC */}
              <div className="flex items-center overflow-hidden relative -left-8">
                <div
                  className="flex flex-row-reverse gap-8 items-center transition-transform duration-500 ease-in-out"
                  style={{ transform: `translateX(${-btcTranslateX}px)` }}
                >
                  {getImages(btcStart, BTC_IMAGE_COUNT).map((num, index) => (
                    <img
                      key={`btc-${num}-${index}`}
                      src={getImagePath('btc', num)}
                      className="w-[191px] flex-shrink-0 cursor-pointer"
                      onClick={() => {
                        window.open(getImageUrl('btc', num - 1), '_blank')
                      }}
                    />
                  ))}
                </div>
              </div>

              {/* Move */}
              <div className="flex items-center overflow-hidden">
                <div
                  className="flex gap-8 items-center transition-transform duration-500 ease-in-out"
                  style={{ transform: `translateX(${moveTranslateX}px)` }}
                >
                  {getImages(moveStart, MOVE_IMAGE_COUNT).map((num, index) => (
                    <img
                      key={`move-${num}-${index}`}
                      src={getImagePath('move', num)}
                      className="w-[191px] flex-shrink-0 cursor-pointer"
                      onClick={() => {
                        window.open(getImageUrl('move', num - 1), '_blank')
                      }}
                    />
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="relative">
          <style jsx>{`
            @keyframes scrollRight {
              from {
                background-position: 0 0;
              }
              to {
                background-position: 100% 0;
              }
            }
          `}</style>
          <div
            className="relative z-10"
            style={{
              background: 'url(./home/cation.svg)',
              animation: 'scrollRight 20s linear infinite',
              width: '100vw',
              height: '64px',
              backgroundRepeat: 'repeat-x',
            }}
          ></div>
        </div>

        {/* BLOG */}
        <div className="py-16 md:py-20 px-4 sm:px-6 md:px-8 lg:px-20 dark:bg-inherit flex flex-col items-center justify-center gap-6 md:gap-8">
          <div className="px-4 w-full h-full">
            <div className="flex flex-col items-center justify-center w-full font-['Han']">
              <div className="mt-14 text-5xl md:text-5xl font-bold text-center text-black dark:text-[#EEEBEB]">
                Blog & News
              </div>
            </div>
          </div>
          <div className="w-full flex items-center justify-center font-[Kanit]">
            <div
              className="w-[45%] h-[586px] border-2 border-[#036840] rounded-[15px] cursor-pointer bg-white dark:bg-inherit"
              onClick={() => {
                window.open('https://rooch.network/blog/rooch-network-tokenomics', '_blank')
              }}
            >
              <div className="w-full h-[376px] -mt-[2px]">
                <img
                  src="/blog/rooch-tokenomics/img1.png"
                  className="inline-block w-full h-full object-cover rounded-t-[15px] mt-1"
                  alt="blog-1"
                />
              </div>
              <div className="flex flex-col items-start p-6">
                <div className="text-base font-normal font-[Kanit]">22th Jan, 2025</div>
                <div className="w-[80%] text-4xl mt-6 font-normal font-[Kanit]">
                  Rooch Network Tokenomics - $ROOCH
                </div>
              </div>
            </div>
            <div className="w-[45%] flex flex-col gap-y-2 ml-12">
              <div
                className="w-full h-[190px] border-2 border-[#036840] rounded-[15px] flex p-6 cursor-pointer bg-white dark:bg-inherit"
                onClick={() => {
                  window.open('https://rooch.network/blog/rooch-ambassador-program', '_blank')
                }}
              >
                <div className="w-[40%] max-w-[220px] h-full flex flex-col items-center">
                  <img
                    src="/blog/ambassador.png"
                    alt="blog-1"
                    className="w-full rounded-xl object-cover"
                  />
                  <div className="text-base font-normal mt-2 font-[Kanit]">14th Feb, 2025</div>
                </div>
                <div className="w-[60%] h-full flex flex-col justify-between ml-4">
                  <div className="text-2xl font-normal font-[Kanit]">
                    Announcing The Rooch Network Ambassador Program
                  </div>
                  <div className="ml-auto">
                    <img src="/home/blog-enter.svg" className="h-8" alt="blog-enter" />
                  </div>
                </div>
              </div>
              <div
                className="w-full h-[190px] border-2 border-[#036840] rounded-[15px] flex p-6 cursor-pointer bg-white dark:bg-inherit"
                onClick={() => {
                  window.open(
                    'https://rooch.network/blog/partnership-annoucement-pan-network',
                    '_blank',
                  )
                }}
              >
                <div className="w-[40%] max-w-[220px] h-full flex flex-col items-center">
                  <img
                    src="/blog/partnership/pan/pan.png"
                    alt="blog-2"
                    className="w-full rounded-xl object-cover"
                  />
                  <div className="text-base font-normal mt-2">10th Feb, 2025</div>
                </div>
                <div className="w-[60%] h-full flex flex-col justify-between ml-4">
                  <div className="text-2xl font-normal">
                    Rooch Network x PAN Network Dual Mining Program
                  </div>
                  <div className="ml-auto">
                    <img src="/home/blog-enter.svg" className="h-8" alt="blog-enter" />
                  </div>
                </div>
              </div>
              <div
                className="w-full h-[190px] border-2 border-[#036840] rounded-[15px] flex p-6 cursor-pointer bg-white dark:bg-inherit"
                onClick={() => {
                  window.open('https://rooch.network/blog/partnership-annoucement-world3', '_blank')
                }}
              >
                <div className="w-[40%] max-w-[220px] h-full flex flex-col items-center">
                  <img
                    src="/blog/partnership/world3/world3.png"
                    alt="blog-3"
                    className="w-full rounded-xl object-cover"
                  />
                  <div className="text-base font-normal mt-2">14th Nov, 2024</div>
                </div>
                <div className="w-[60%] h-full flex flex-col justify-between ml-4">
                  <div className="text-2xl font-normal">
                    Rooch Network x World3 Dual Mining Program{' '}
                  </div>
                  <div className="ml-auto">
                    <img src="/home/blog-enter.svg" className="h-8" alt="blog-enter" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="relative">
          <style jsx>{`
            @keyframes scrollRight {
              from {
                background-position: 0 0;
              }
              to {
                background-position: 100% 0;
              }
            }
          `}</style>
          <div
            className="relative z-10"
            style={{
              background: 'url(./home/cation.svg)',
              animation: 'scrollRight 20s linear infinite',
              width: '100vw',
              height: '64px',
              backgroundRepeat: 'repeat-x',
            }}
          ></div>
        </div>
      </div>
    </>
  )
}

export default Index
