import { DocsThemeConfig } from 'nextra-theme-docs'
import { Footer } from './components/layout/footer'
import Image from 'next/image'
import { useRouter } from 'next/router'
import { useConfig } from 'nextra-theme-docs'

const theme: DocsThemeConfig = {
  docsRepositoryBase: 'https://github.com/rooch-network/rooch/blob/main/docs/website',
  nextThemes: {
    defaultTheme: 'system',
  },
  logo: (
    <div>
      <Image
        src="/logo/rooch_black_combine.svg"
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
    if (asPath !== '/') {
      if (asPath.includes('/docs/')) {
        return {
          titleTemplate: '%s – Rooch Network Documentation',
        }
      }
      return {
        titleTemplate: '%s – Rooch Network',
      }
    } else {
      return {
        titleTemplate: '%s',
      }
    }
  },
  head: function useHead() {
    const { title, frontMatter } = useConfig()
    const { asPath } = useRouter()
    const router = useRouter()
    // const socialCard = '/logo/rooch-banner.png'
    const currentLang = router.locale
    const pageDescription = frontMatter.description
      ? frontMatter.description
      : currentLang === 'en-US'
      ? 'Unlocking infinite utility for the Bitcoin Economy'
      : '开启比特币经济的无限可能'
    return (
      <>
        <meta name="msapplication-TileColor" content="#ffffff" />
        <meta name="theme-color" content="#ffffff" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        {/* MULTI-LANGUAGES */}
        <link rel="alternate" href={'https://rooch.network' + asPath} hrefLang="x-default" />
        <link rel="alternate" href={'https://rooch.network' + asPath} hrefLang="en-us" />
        <link rel="alternate" href={'https://rooch.network' + asPath} hrefLang="en" />
        <link rel="alternate" href={'https://rooch.network/zh-CN' + asPath} hrefLang="zh-cn" />
        <link rel="alternate" href={'https://rooch.network/zh-CN' + asPath} hrefLang="zh" />
        {/* WEBSITE */}
        <meta name="description" content={pageDescription} />
        <meta property="og:description" content={pageDescription} />
        <meta property="og:image" content="https://rooch.network/logo/rooch-banner.png" />
        <meta name="apple-mobile-web-app-title" content="Rooch Network" />
        {/* TWITTER */}
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:site" content="https://rooch.network" />
        <meta name="twitter:creator" content="https://rooch.network" />
        <meta
          name="twitter:title"
          content="Rooch Network | Unlocking Infinite Utility for the Bitcoin Economy"
        />
        <meta name="twitter:description" content={pageDescription} />
        <meta name="twitter:image" content="https://rooch.network/logo/rooch-banner.png" />
        <meta name="twitter:image:alt" content="Rooch Network" />
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
