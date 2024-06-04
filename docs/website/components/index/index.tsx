import Link from 'next/link'
import Image from 'next/image'

interface Card {
  title: string
  description: string
  logo: string
  buttonHref?: string
  buttonDesc?: string
}

interface Brand {
  brandLogo: string
  brandTitle: string
  brandUrl: string
}

interface Blog {
  title: string
  date: string
  link: string
  image: string
}

interface IndexProps {
  // HERO
  heroTitle: string
  heroSlogan?: string
  heroDescription: string
  heroButton: string
  heroButtonHref: string

  // FEATURES
  featuresTitle: string
  featuresButton: string
  features: Card[]

  // EXPLORE
  exploreTitle: string
  exploreContent: string
  exploreButtonHref: string
  explores: Card[]

  // ECOSYSTEM
  ecosystemTitle: string
  ecosystemContent: string
  ecosystemBrand: Brand[]

  // BLOGS
  blogsTitle: string
  blogs: Blog[]
}

const Index = ({
  heroTitle,
  heroSlogan,
  heroDescription,
  heroButton,
  heroButtonHref,
  featuresTitle,
  featuresButton,
  features,
  exploreTitle,
  exploreContent,
  exploreButtonHref,
  explores,
  ecosystemTitle,
  // ecosystemContent,
  ecosystemBrand,
  blogsTitle,
  blogs,
}: IndexProps) => {
  // Function to check if the string contains Chinese characters
  const containsChinese = (text: string) => /[\u4e00-\u9fa5]/.test(text)

  // Define phrases to highlight for Chinese text
  const phrasesToHighlightForFeaturesChinese = ['比特币生态']
  const phrasesToHighlightForExploreChinese = ['状态', '应用']
  const phrasesToHighlightForEcosystemChinese = ['合作伙伴']
  const phrasesToHighlightForBlogsChinese = ['博客']

  // Define phrases to highlight for English text
  const phrasesToHighlightForFeaturesEnglish = ['Bitcoin', 'Ecosystem']
  const phrasesToHighlightForExploreEnglish = ['State', 'App']
  const phrasesToHighlightForEcosystemEnglish = ['Partnerships']
  const phrasesToHighlightForBlogsEnglish = ['Blog']

  const highlightColor = '#FF914B'
  const highlightColorForExplore = '#46977E'

  const highlightSpecificPhrases = (
    text: string,
    phrasesToHighlight: string[],
    highlightColor: string,
  ) => {
    let result = text
    phrasesToHighlight.forEach((phrase) => {
      const regex = new RegExp(`(${phrase})`, 'g')
      result = result.replace(regex, `<span style="color: ${highlightColor};">$1</span>`)
    })
    return <span dangerouslySetInnerHTML={{ __html: result }} />
  }

  const highlightTitle = (
    title: string,
    phrasesToHighlightChinese: string[],
    phrasesToHighlightEnglish: string[],
    highlightColor: string,
  ) => {
    if (containsChinese(title)) {
    } else {
      const words = title.split(' ')
      return (
        <>
          {words.map((word, index) =>
            phrasesToHighlightEnglish.includes(word) ? (
              <span key={index} style={{ color: highlightColor }}>
                {word}{' '}
              </span>
            ) : (
              word + ' '
            ),
          )}
        </>
      )
    }
  }

  const handleButtonOnClick = (href: string) => {
    console.log(href)
    window.open(href)
  }

  return (
    <>
      <div className="antialiased">
        {/* HERO */}
        <div className="mt-28 md:mt-0 flex flex-col md:flex-row items-center justify-center md:justify-between h-full md:h-[90vh] px-4 sm:px-6 md:px-8 lg:px-20 dark:border-b dark:border-b-zinc-800">
          <div className="flex flex-col items-center justify-center">
            <div className="mt-5 max-w-3xl text-center mx-auto">
              <p className="block font-bold text-gray-800 text-4xl md:text-5xl lg:text-6xl dark:text-gray-200">
                {heroTitle}
              </p>
              {heroSlogan ? (
                <p className="block pt-2 font-bold text-gray-800 text-2xl md:text-3xl lg:text-4xl dark:text-gray-200">
                  {heroSlogan}
                </p>
              ) : null}
            </div>
            <div className="mt-5 max-w-xl text-center mx-auto">
              <p className="text-lg text-gray-600 dark:text-gray-400">{heroDescription}</p>
            </div>
            <div className="mt-8 grid gap-3 w-full sm:inline-flex sm:justify-center cta-container">
              <Link
                className="inline-flex justify-center items-center gap-x-3 text-center bg-gradient-to-tl border border-transparent text-sm font-medium rounded-md py-3 px-6 cta"
                href={heroButtonHref}
              >
                {heroButton}
                <svg className="w-3 h-3" width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <path
                    d="M5.27921 2L10.9257 7.64645C11.1209 7.84171 11.1209 8.15829 10.9257 8.35355L5.27921 14"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                  />
                </svg>
              </Link>
            </div>
          </div>
          <div className="md:w-1/2 w-full my-12 md:my-0">
            <img src="/logo/hero/hero.svg" alt="hero" />
          </div>
        </div>

        {/* FEATURES */}
        <div className="py-16 md:py-20 px-4 sm:px-6 md:px-8 lg:px-20 bg-[#F5F5F5] dark:bg-inherit flex flex-col md:flex-row items-center justify-between gap-12 md:gap-8 dark:border-b dark:border-b-zinc-800">
          <div className="px-4 max-w-[900px] w-full h-full">
            <h2 className="text-4xl md:text-6xl font-semibold text-center md:text-start text-[#2E2929] dark:text-[#EEEBEB]">
              {highlightTitle(
                featuresTitle,
                phrasesToHighlightForFeaturesChinese,
                phrasesToHighlightForFeaturesEnglish,
                highlightColor,
              )}
            </h2>
            <div className="mt-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
              {features?.map((feature) => (
                <div
                  key={feature.title}
                  className="flex flex-col items-center md:items-start justify-center md:justify-start space-y-3 bg-white dark:bg-[#333] p-6 rounded-2xl shadow-md overflow-hidden h-full hover:cursor-default"
                >
                  <div className="w-12 h-12 md:w-16 md:h-16 mb-4">
                    <Image
                      src={feature.logo}
                      alt="features logo"
                      width={50}
                      height={50}
                      className="dark:filter dark:invert dark:brightness-150"
                    />
                  </div>
                  <h3 className="md:text-2xl text-[#FF914B] text-center md:text-start mb-2 font-bold text-2xl leading-7">
                    {feature.title}
                  </h3>
                  <div className="text-gray-600 text-center md:text-start dark:text-[#EAEAEA]">
                    <p>{feature.description}</p>
                  </div>
                  <div className="flex items-end justify-end h-full w-full">
                    <div
                      className="font-semibold text-[#2E2929] dark:text-[#EEEBEB] hover:cursor-pointer flex items-center justify-center space-x-2 hover:underline transition-all"
                      onClick={() => handleButtonOnClick(feature.buttonHref)}
                    >
                      <span>{feature.buttonDesc}</span>
                      <svg
                        className="w-3 h-3"
                        width="16"
                        height="16"
                        viewBox="0 0 16 16"
                        fill="none"
                      >
                        <path
                          d="M5.27921 2L10.9257 7.64645C11.1209 7.84171 11.1209 8.15829 10.9257 8.35355L5.27921 14"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                        />
                      </svg>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
          <div className="flex flex-wrap justify-center items-center w-full md:w-auto">
            <img
              src="/logo/features/features_logo.svg"
              alt="features logo"
              className="w-full h-auto object-cover"
            />
          </div>
        </div>

        {/* EXPLORE */}
        <div className="py-20 px-4 sm:px-6 md:px-8 lg:px-20 bg-white dark:bg-inherit flex flex-col md:flex-row items-center justify-center gap-6 md:gap-8 dark:border-b dark:border-b-zinc-800">
          <div className="px-4 max-w-[854px] w-full h-full">
            <h2 className="text-4xl md:text-6xl font-semibold text-center text-[#2E2929] dark:text-[#E4E4E4]">
              {highlightTitle(
                exploreTitle,
                phrasesToHighlightForExploreChinese,
                phrasesToHighlightForExploreEnglish,
                highlightColorForExplore,
              )}
            </h2>
            <div className="flex flex-col items-center justify-center md:justify-start gap-6 mt-8">
              <h3 className="text-[#737B7D] dark:text-[#81888A] text-base font-normal max-w-2xl text-center">
                {exploreContent}
              </h3>
              <img
                src="/logo/explore/explore_logo.svg"
                alt="explore logo"
                className="w-full md:w-9/12 h-full dark:hidden block"
              />
              <img
                src="/logo/explore/explore_logo_dark.svg"
                alt="explore logo"
                className="w-full md:w-9/12 h-full hidden dark:block"
              />
              <div className="mt-8 h-12">
                <button
                  className="px-8 py-4 bg-[#FF914B] font-bold text-lg text-center rounded-full border border-1 border-b-[6px] border-black active:border-b-4 active:transform active:translate-y-0.5 hover:shadow-custom1 dark:border-white dark:shadow-custom1 transition-all"
                  onClick={() => handleButtonOnClick(exploreButtonHref)}
                >
                  {featuresButton}
                </button>
              </div>
            </div>
          </div>

          {/* EXPLORE CARDS */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8 md:mt-0 w-full">
            {explores.map((explore) => (
              <div
                key={explore.title}
                className="bg-[#BFC9C6] rounded-2xl border border-black p-6 flex flex-col items-start justify-center gap-4 hover:cursor-default hover:shadow-lg transition-shadow duration-300 h-full"
              >
                <div className="flex items-center justify-end w-full mb-4">
                  <Image src={explore.logo} alt={explore.title} width={100} height={100} />
                </div>
                <h4 className="text-[#413434] font-bold text-2xl leading-7">{explore.title}</h4>
                <p className="text-[#413434]">{explore.description}</p>
              </div>
            ))}
          </div>
        </div>

        {/* ECOSYSTEM */}
        <div className="py-20 px-4 sm:px-6 md:px-8 lg:px-20 flex flex-col items-center justify-center gap-6 md:gap-0 bg-[#F5F5F5] dark:bg-inherit w-full dark:border-b dark:border-b-zinc-800">
          <div className="flex flex-col items-center justify-center h-full gap-2">
            <h2 className="text-4xl md:text-6xl font-semibold text-center md:text-start text-[#2E2929] dark:text-[#E6E6E6]">
              {highlightTitle(
                ecosystemTitle,
                phrasesToHighlightForEcosystemChinese,
                phrasesToHighlightForEcosystemEnglish,
                highlightColor,
              )}
            </h2>
            <p className="mt-4 text-[#737B7D] dark:text-[#81888A] text-center md:text-start">
              {/*{ecosystemContent}*/}
            </p>
          </div>
          <div className="flex items-center justify-center w-full mt-2">
            <div className="grid grid-cols-2 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 justify-items-center">
              {ecosystemBrand.map((brand) => (
                <button
                  className="relative bg-white dark:bg-inherit rounded-full md:rounded-lg p-4 flex flex-row md:flex-col items-center md:justify-center justify-start border border-1 border-b-[6px] border-black dark:border-white active:border-b-4 active:transform active:translate-y-0.5 transition-all shadow-sm w-full h-14 md:w-52 md:h-32 gap-2 hover:cursor-pointer hover:shadow-md"
                  onClick={() => handleButtonOnClick(brand.brandUrl)}
                >
                  <Image
                    src={brand.brandLogo}
                    alt={brand.brandTitle}
                    width={60}
                    height={60}
                    className="w-[25px] h-[25px] md:w-[60px] md:h-[60px]"
                  />
                  <p className="text-center text-base font-semibold dark:text-zinc-200">
                    {brand.brandTitle}
                  </p>
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* BLOG */}
        <div className="py-16 md:py-20 px-4 sm:px-6 md:px-8 lg:px-20 bg-white dark:bg-inherit flex flex-col md:flex-row items-center justify-center gap-6 md:gap-8">
          <div className="px-4 w-full h-full">
            <h2 className="text-4xl md:text-6xl font-semibold text-center md:text-start text-[#2E2929] dark:text-[#E9E9E9]">
              {highlightTitle(
                blogsTitle,
                phrasesToHighlightForBlogsChinese,
                phrasesToHighlightForBlogsEnglish,
                highlightColor,
              )}
            </h2>
            <div className="mt-8 flex flex-col gap-8">
              {blogs?.map((blog) => (
                <Link key={blog.title} href={blog.link} className="block">
                  <div className="bg-inherit md:bg-white dark:bg-[#333] shadow-md hover:shadow-lg dark:hover:border-[#555] rounded-lg md:border border-gray-200 dark:border-[#333] transition-all duration-300 flex flex-col md:flex-row overflow-hidden">
                    <div className="md:w-1/3 w-full h-auto">
                      <img
                        src={blog.image}
                        alt={blog.title}
                        className="h-[200px] w-full object-fill rounded-lg"
                      />
                    </div>
                    <div className="flex flex-col p-6 md:p-8 w-full justify-between">
                      <div>
                        <h3 className="text-2xl md:text-3xl font-semibold text-gray-800 dark:text-zinc-200">
                          {blog.title}
                        </h3>
                        <p className="text-gray-600 mt-2 text-sm dark:text-zinc-400">{blog.date}</p>
                      </div>
                      <button className="mt-4 self-end text-blue-500 hover:text-blue-700 transition-colors">
                        <Image
                          src="/logo/blogs/chevron_right.svg"
                          alt="Chevron right"
                          width={24}
                          height={24}
                          className="dark:filter dark:invert"
                        />
                      </button>
                    </div>
                  </div>
                </Link>
              ))}
            </div>
          </div>
        </div>
      </div>
    </>
  )
}

export default Index
