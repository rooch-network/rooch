import Link from 'next/link'
import Image from 'next/image'

interface Card {
  title: string
  description: string
  logo: string
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
  features: Card[]

  // EXPLORE
  exploreTitle: string
  exploreContent: string
  explores: Card[]

  // ECOSYSTEM
  ecosystemTitle: string
  ecosystemContent: string

  // BLOGS
  blogsTitle: string
  blogs: { title: string; date: string; link: string }[]
}

export default function Index({
  heroTitle,
  heroSlogan,
  heroDescription,
  heroButton,
  heroButtonHref,
  featuresTitle,
  features,
  exploreTitle,
  exploreContent,
  explores,
  ecosystemTitle,
  ecosystemContent,
  blogsTitle,
  blogs,
}: IndexProps) {
  return (
    <>
      {/* HERO */}
      <div className="mx-auto px-4 sm:px-6 lg:px-8 pt-60 pb-60">
        {/* <!-- Title --> */}
        <div className="mt-5 max-w-3xl text-center mx-auto">
          <p className="block font-bold text-gray-800 text-4xl md:text-5xl lg:text-5xl dark:text-gray-200">
            {heroTitle}
          </p>
          {heroSlogan ? (
            <p className="block pt-2 font-bold text-gray-800 text-2xl md:text-3xl lg:text-4xl dark:text-gray-200">
              {heroSlogan}
            </p>
          ) : (
            ''
          )}
        </div>
        {/* <!-- End Title --> */}

        <div className="mt-5 max-w-2xl text-center mx-auto">
          <p className="text-lg text-gray-600 dark:text-gray-400">{heroDescription}</p>
        </div>

        {/* <!-- Buttons --> */}
        <div className="mt-8 grid gap-3 w-full sm:inline-flex sm:justify-center cta-container">
          <Link
            className="inline-flex justify-center items-center gap-x-3 text-center bg-gradient-to-tl border border-transparent text-sm font-medium rounded-md  py-3 px-6 cta"
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
        {/* <!-- End Buttons --> */}
      </div>

      {/* FEATURES */}
      <div className="py-16 md:py-32 px-6 md:px-24 bg-[#F5F5F5] flex flex-col md:flex-row items-center justify-center gap-6 md:gap-0">
        <div className="px-4 max-w-[854px]">
          <h2 className="text-4xl md:text-6xl font-semibold text-center md:text-start text-[#2E2929]">
            {featuresTitle.split(' ').slice(0, -1).join(' ')}{' '}
            <span className="text-[#FF914B]">{featuresTitle.split(' ').slice(-1)}</span>
          </h2>
          <div className="mt-16 md:mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
            {features?.map((feature, index) => (
              <div
                key={index}
                className="flex flex-col items-center justify-center md:items-start md:justify-start"
              >
                <div className="w-12 h-12 md:w-16 md:h-16">
                  <Image src={feature.logo} alt="features logo" width={40} height={40} />
                </div>
                <h3 className="text-4xl font-medium text-[#FF914B] text-center md:text-start">
                  {feature.title}
                </h3>
                <p className="text-gray-600 mt-2 text-center md:text-start">
                  {feature.description}
                </p>
              </div>
            ))}
          </div>
        </div>
        <div>
          <Image
            src="/logo/features/features_logo.svg"
            alt="features logo"
            width={433}
            height={410}
          />
        </div>
      </div>

      {/* EXPLORE */}
      <div className="py-16 md:py-32 px-6 md:px-24 bg-white flex flex-col md:flex-row items-center justify-center gap-6 md:gap-0">
        <div className="px-4 max-w-[854px] h-full w-full">
          <h2 className="text-4xl md:text-6xl font-semibold text-center md:text-start text-[#2E2929]">
            {exploreTitle.split(' ').slice(0, -2).join(' ')}{' '}
            <span className="text-[#46977E]">{exploreTitle.split(' ').slice(-2).join(' ')}</span>
          </h2>
          <div className="flex flex-col items-center justify-center md:items-start md:justify-start gap-6">
            <h3 className="text-[#737B7D] text-base font-normal max-w-2xl">{exploreContent}</h3>
            <Image
              src="/logo/explore/explore_logo.svg"
              alt="explore logo"
              width={280}
              height={280}
            />
            <div className="mt-16">
              <Image
                src="/logo/explore/explore_button.svg"
                alt="explore button"
                width={360}
                height={60}
                className="hover:cursor-pointer overflow-hidden hover:opacity-80 transition-all"
              />
            </div>
          </div>
        </div>

        {/* EXPLORE CARDS */}
        <div className="flex flex-wrap justify-center gap-4">
          {explores.map((explore) => (
            <div className="bg-[#BFC9C6] rounded-2xl border border-1 border-black shadow-lg p-6 w-full md:w-60 md:h-85 flex flex-col items-start justify-center gap-1">
              <div className="flex items-center justify-end w-full">
                <Image src={explore.logo} alt={explore.title} width={100} height={100} />
              </div>
              <h4 className="text-[#413434] font-bold text-2xl">{explore.title}</h4>
              <h4 className="text-[#413434] text-sm">{explore.description}</h4>
            </div>
          ))}
        </div>
      </div>

      {/* ECOSYSTEM */}
      <div className="py-20 bg-gray-100">
        <div className="container mx-auto px-4 text-center">
          <h2 className="text-3xl font-bold text-gray-800">{ecosystemTitle}</h2>
          <p className="text-gray-600 mt-4">{ecosystemContent}</p>
        </div>
      </div>

      {/* BLOGS */}
      <div className="py-20">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl font-bold text-center text-gray-800">{blogsTitle}</h2>
          <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
            {blogs?.map((blog, index) => (
              <div key={index} className="bg-white shadow-lg rounded-lg p-6">
                <h3 className="text-xl font-bold text-gray-800">{blog.title}</h3>
                <p className="text-gray-600 mt-2">{blog.date}</p>
                <Link href={blog.link}>
                  <button className="text-blue-500 hover:text-blue-700 mt-4 inline-block">
                    Read More
                  </button>
                </Link>
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
  )
}
