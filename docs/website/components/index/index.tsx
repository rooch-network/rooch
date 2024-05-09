import Link from 'next/link'

interface Feature {
  title: string
  description: string
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
  features: Feature[]

  // EXPLORE
  exploreTitle: string
  exploreContent: string

  // ECOSYSTEM
  ecosystemTitle: string
  ecosystemContent: string

  // BLOGS
  blogsTitle: string
  blogs: { title: string; summary: string; link: string }[]
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
      <div className="py-20 bg-gray-100">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl font-bold text-center text-gray-800">{featuresTitle}</h2>
          <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
            {features?.map((feature, index) => (
              <div key={index} className="bg-white shadow-lg rounded-lg p-6">
                <h3 className="text-xl font-bold text-gray-800">{feature.title}</h3>
                <p className="text-gray-600 mt-2">{feature.description}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* EXPLORE */}
      <div className="py-20">
        <div className="container mx-auto px-4 text-center">
          <h2 className="text-3xl font-bold text-gray-800">{exploreTitle}</h2>
          <p className="text-gray-600 mt-4">{exploreContent}</p>
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
                <p className="text-gray-600 mt-2">{blog.summary}</p>
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
