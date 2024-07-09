import Image from 'next/image'
import { usePathname } from 'next/navigation'
import { getPagesUnderRoute } from 'nextra/context'
import ROOCH_TEAM from '../../data/team'
import { useState, useEffect } from 'react'
import Head from 'next/head'

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
  return page && page.frontMatter && typeof page.frontMatter.title === 'string'
}

export default function PostHeader() {
  const pathname = usePathname()
  const [page, setPage] = useState<CustomPage | null>(null)
  const [ogImage, setOgImage] = useState('https://rooch.network/logo/rooch-banner.png')

  const fetchPage = () => {
    const pages = getPagesUnderRoute('/blog') as unknown as any[]
    const customPages = pages.filter(isCustomPage) as CustomPage[]
    const currentPage = customPages.find((page) => page.route === pathname)
    if (currentPage) {
      setPage(currentPage)
      if (currentPage.frontMatter.image) {
        setOgImage(`https://rooch.network${currentPage.frontMatter.image}`)
      }
    }
  }
  useEffect(() => {
    fetchPage()
  }, [pathname])

  return page ? (
    <>
      <Head>
        <title>{page.frontMatter.title}</title>
        <meta property="og:title" content={page.frontMatter.title} />
        <meta property="og:description" content={page.frontMatter.description} />
        <meta property="og:image" content={ogImage} />
        <meta property="og:type" content="article" />
        <meta name="twitter:card" content="summary" />
        <meta name="twitter:title" content={page.frontMatter.title} />
        <meta name="twitter:image" content={ogImage} />
      </Head>
      <div className="text-center inline-block mx-auto w-full">
        <h1 className="font-bold text-5xl mt-6">{page.frontMatter.title}</h1>
        <h2 className="my-3 text-sm inline-flex gap-2 uppercase text-gray-500 dark:text-gray-300">
          {page.frontMatter.category}
          {' | '}
          <Image
            src={ROOCH_TEAM[page.frontMatter.author].avatar}
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
