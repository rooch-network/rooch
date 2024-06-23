import Image from 'next/image'
import { usePathname } from 'next/navigation'
import { getPagesUnderRoute } from 'nextra/context'
import ROOCH_TEAM from '../../data/team'
import { useState, useEffect, useCallback } from 'react'
import Head from 'next/head'

interface FrontMatter {
  title: string
  description: string
  image?: string
  category: string
  author: string
  meta?: {
    title?: string
    description?: string
    image?: string
  }
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

  const fetchPage = useCallback(() => {
    try {
      const pages = getPagesUnderRoute('/blog') as any[]
      const customPages = pages.filter(isCustomPage) as CustomPage[]
      const currentPage = customPages.find((page) => page.route === pathname)
      console.log('currentPage', currentPage)
      if (currentPage) {
        setPage(currentPage)
        const metaImage = currentPage.frontMatter.meta?.image || currentPage.frontMatter.image
        if (metaImage) {
          setOgImage(metaImage.startsWith('http') ? metaImage : `https://rooch.network${metaImage}`)
        }
      }
    } catch (error) {
      console.error('Failed to fetch page:', error)
    }
  }, [pathname])

  useEffect(() => {
    fetchPage()
  }, [fetchPage])

  const frontMatterMeta = page?.frontMatter.meta || {}
  const title = frontMatterMeta.title || page?.frontMatter.title
  const description = frontMatterMeta.description || page?.frontMatter.description
  const image = frontMatterMeta.image || ogImage

  console.log(frontMatterMeta)

  return page ? (
    <>
      <Head>
        <title>{title}</title>
        <meta property="og:title" content={title} />
        <meta property="og:description" content={description} />
        <meta property="og:image" content={image} />
        <meta property="og:type" content="article" />
        <meta property="og:url" content={`https://rooch.network${pathname}`} />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content={title} />
        <meta name="twitter:description" content={description} />
        <meta name="twitter:image" content={image} />
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
