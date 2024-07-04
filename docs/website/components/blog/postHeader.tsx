import Image from 'next/image'
import Head from 'next/head'
import { getPagesUnderRoute } from 'nextra/context'
import ROOCH_TEAM from '../../data/team'

interface FrontMatter {
  title: string
  description: string
  image?: string
  category: string
  author: string
}

interface CustomPage {
  route: string
  frontMatter: FrontMatter
}

function isCustomPage(page: any): page is CustomPage {
  return (
    typeof page === 'object' &&
    page !== null &&
    'frontMatter' in page &&
    typeof page.frontMatter === 'object' &&
    typeof page.frontMatter.title === 'string'
  )
}

export default function PostHeader({ page, ogImage }: { page: CustomPage; ogImage: string }) {
  return page ? (
    <>
      <Head>
        <title>{page.frontMatter.title}</title>
        <meta property="og:title" content={page.frontMatter.title} />
        <meta property="og:description" content={page.frontMatter.description} />
        <meta property="og:image" content={ogImage} />
        <meta property="og:type" content="article" />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content={page.frontMatter.title} />
      </Head>
      <div className="text-center inline-block mx-auto w-full">
        <h1 className="font-bold text-5xl mt-6">{page.frontMatter.title}</h1>
        <h2 className="my-3 text-sm inline-flex gap-2 uppercase text-gray-500 dark:text-gray-300">
          {page.frontMatter.category}
          {' | '}
          <Image
            src={page.frontMatter.image || '/default-avatar.png'}
            alt={page.frontMatter.author}
            width={20}
            height={20}
            className="rounded-full"
          />
          <span className="font-semibold">
            <a
              href={'https://twitter.com/' + ROOCH_TEAM[page.frontMatter.author].twitterUsername}
              target="_blank"
              rel="noopener noreferrer"
            >
              {ROOCH_TEAM[page.frontMatter.author].name}
            </a>
          </span>
        </h2>
      </div>
    </>
  ) : null
}

export async function getStaticProps(context: any) {
  const pages = (await getPagesUnderRoute('/blog')) as unknown[]
  const customPages = pages.filter(isCustomPage)
  const currentPage = customPages.find((page) => page.route === context.params.path)

  return {
    props: {
      page: currentPage,
      ogImage:
        currentPage && currentPage.frontMatter.image
          ? `https://rooch.network${currentPage.frontMatter.image}`
          : 'https://rooch.network/logo/rooch-banner.png',
    },
  }
}

export async function getStaticPaths() {
  const pages = (await getPagesUnderRoute('/blog')) as unknown[]
  const paths = pages.filter(isCustomPage).map((page) => ({ params: { path: page.route } }))

  return { paths, fallback: false }
}
