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
    ];
  },
});
