import { useEffect, useState } from 'react'
import { DocsThemeConfig } from 'nextra-theme-docs'
import { Footer } from './components/layout/footer'
import Image from 'next/image'
import { useRouter } from 'next/router'
import { useConfig } from 'nextra-theme-docs'
import { getPagesUnderRoute } from 'nextra/context'

interface Page {
  kind: 'MdxPage'
  name: string
  route: string
  frontMatter: {
    title: string
    description: string
    date: string
    author: string
    category: string
    image?: string
  }
}

interface Folder {
  kind: 'Folder'
  name: string
  children: (Page | Folder)[]
}

type Content = Page | Folder

const isPage = (content: Content): content is Page => content.kind === 'MdxPage'

const theme: DocsThemeConfig = {
  docsRepositoryBase: 'https://github.com/rooch-network/rooch/blob/main/docs/website',
  nextThemes: {
    defaultTheme: 'system',
  },
  logo: (
    <div>
      <Image
        src="/logo/combine/rooch_black_combine.svg"
        alt="Rooch Architecture"
        width={100}
        height={70}
        className="dark:hidden"
      />
      <Image
        src="/logo/rooch_white_combine.svg"
        alt="Rooch Architecture"
        width={100}
        height={70}
        className="hidden dark:block"
      />
    </div>
  ),
  useNextSeoProps() {
    const { asPath } = useRouter()
    return {
      titleTemplate: asPath.includes('/docs/')
        ? '%s – Rooch Network Documentation'
        : '%s – Rooch Network',
    }
  },
  head: function useHead() {
    const { title, frontMatter } = useConfig()
    const { asPath } = useRouter()
    const router = useRouter()
    const currentLang = router.locale

    const defaultDescription =
      currentLang === 'en-US'
        ? 'Unlocking infinite utility for the Bitcoin Economy'
        : '开启比特币经济的无限可能'

    const [pageTitle, setPageTitle] = useState(title || 'Rooch Network')
    const [pageDescription, setPageDescription] = useState(frontMatter.description || '')
    const [ogImage, setOgImage] = useState('https://rooch.network/logo/rooch-banner.png')

    useEffect(() => {
      if (asPath.includes('/blog/')) {
        const contents = getPagesUnderRoute('/blog') as Content[]
        const currentPage = contents.find(
          (content): content is Page => isPage(content) && content.route === asPath,
        )
        if (currentPage) {
          setPageTitle(
            currentPage.frontMatter.title
              ? `${currentPage.frontMatter.title} – Rooch Network`
              : 'Rooch Network',
          )
          setPageDescription(currentPage.frontMatter.description || '')
          setOgImage(
            currentPage.frontMatter.image
              ? `https://rooch.network${currentPage.frontMatter.image}`
              : 'https://rooch.network/logo/rooch-banner.png',
          )
          return
        }
      } else {
        setPageTitle(title || 'Rooch Network')
        setPageDescription(frontMatter.description || defaultDescription)
        setOgImage('https://rooch.network/logo/rooch-banner.png')
      }
    }, [asPath, title, frontMatter, defaultDescription])

    return (
      <>
        <meta name="msapplication-TileColor" content="#ffffff" />
        <meta name="theme-color" content="#ffffff" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        {/* MULTI-LANGUAGES */}
        <link rel="alternate" href={`https://rooch.network${asPath}`} hrefLang="x-default" />
        <link rel="alternate" href={`https://rooch.network${asPath}`} hrefLang="en-us" />
        <link rel="alternate" href={`https://rooch.network${asPath}`} hrefLang="en" />
        <link rel="alternate" href={`https://rooch.network/zh-CN${asPath}`} hrefLang="zh-cn" />
        <link rel="alternate" href={`https://rooch.network/zh-CN${asPath}`} hrefLang="zh" />
        {/* WEBSITE */}
        <meta property="og:type" content="website" />
        <meta property="og:image" content={ogImage} />
        <meta property="og:description" content={pageDescription} />
        <meta name="apple-mobile-web-app-title" content="Rooch Network" />
        {/* TWITTER */}
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:site" content="https://rooch.network" />
        <meta name="twitter:creator" content="https://rooch.network" />
        <meta name="twitter:title" content={pageTitle} />
        {/* FAVICON */}
        <link rel="icon" href="/logo/rooch_black_logo.svg" type="image/svg+xml" />
        <link rel="icon" href="/logo/rooch_black_logo.png" type="image/png" />
        <link
          rel="icon"
          href="/logo/rooch_white_logo.svg"
          type="image/svg+xml"
          media="(prefers-color-scheme: dark)"
        />
        <link
          rel="icon"
          href="/logo/rooch_white_logo.png"
          type="image/png"
          media="(prefers-color-scheme: dark)"
        />
      </>
    )
  },
  project: {
    link: 'https://github.com/rooch-network',
  },
  chat: {
    link: 'https://discord.gg/rooch',
  },
  i18n: [
    { locale: 'en-US', text: 'English' },
    { locale: 'zh-CN', text: '简体中文' },
  ],
  footer: {
    component: Footer,
  },
  sidebar: {
    defaultMenuCollapseLevel: 1,
  },
}

export default theme
