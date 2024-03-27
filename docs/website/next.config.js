const withNextra = require("nextra")({
  theme: "nextra-theme-docs",
  themeConfig: "./theme.config.tsx",
});

module.exports = withNextra({
  i18n: {
    locales: ["en-US", "zh-CN"],
    defaultLocale: "en-US",
  },
  reactStrictMode: true,
  typescript: {
    ignoreBuildErrors: true,
  },
  async redirects() {
    return [
      {
        source: "/docs",
        permanent: false,
        destination: "/docs/introduction",
      },
      {
        source: "/docs/why-rooch",
        permanent: false,
        destination: "/docs/rooch",
      },
      {
        source: "/contact-us",
        permanent: false,
        destination: "/about#contact",
      },
      {
        source: "/docs/technology/move-on-rooch",
        permanent: false,
        destination: "/docs/tech-highlights/move-language",
      },
      {
        source: "/docs/developer-guides/object",
        permanent: false,
        destination: "/learn/core-concepts/objects/object",
      },
      {
        source: "/docs/developer-guides/quick-start",
        permanent: false,
        destination: "/build/getting-started/first-contract/quick-start",
      },
      {
        source: "/docs/developer-guides/create-rooch-move-contract",
        permanent: false,
        destination: "/build/getting-started/first-contract/create-rooch-move-contract",
      },
      {
        source: "/docs/developer-guides/coin-intro",
        permanent: false,
        destination: "/learn/getting-started/first-contract/first-token/coin-intro",
      },
      {
        source: "/docs/getting-started",
        permanent: false,
        destination: "/build/getting-started/first-blog-system",
      },
    ];
  },
});
