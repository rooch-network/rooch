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
        source: "/docs/introduction",
        permanent: false,
        destination: "/learn/introduction",
      },
      {
        source: "/docs/rooch",
        permanent: false,
        destination: "/learn/architecture",
      },
      {
        source: "/docs/rooch-network",
        permanent: false,
        destination: "/learn/architecture",
      },
      {
        source: "/docs/tech-highlights/account-abstraction",
        permanent: false,
        destination: "/learn/core-concepts/accounts/account-abstraction",
      },
      {
        source: "/docs/developer-guides/access-path",
        permanent: false,
        destination: "/learn/core-concepts/accounts/address-space",
      },
      {
        source: "/docs/dive-into-rooch/transaction-flow",
        permanent: false,
        destination: "/learn/core-concepts/transaction/transaction-flow",
      },
      {
        source: "/docs/dive-into-rooch/storage-abstraction",
        permanent: false,
        destination: "/learn/core-concepts/objects/storage-abstraction",
      },
      {
        source: "/docs/developer-guides/move-on-rooch",
        permanent: false,
        destination: "/learn/core-concepts/move-contracts/move-on-rooch",
      },
      {
        source: "/docs/developer-guides/library",
        permanent: false,
        destination: "/learn/core-concepts/move-contracts/built-in-library",
      },
      {
        source: "/docs/tech-highlights/move-language",
        permanent: false,
        destination: "/learn/core-concepts/move-contracts/move-language",
      },
      {
        source: "/docs/tech-highlights/fraud-proof",
        permanent: false,
        destination: "/learn/in-depth-tech/fraud-proof",
      },
      {
        source: "/docs/tech-highlights/hybrid-security",
        permanent: false,
        destination: "/learn/in-depth-tech/hybrid-security",
      },
      {
        source: "/docs/dive-into-rooch/l1-to-l2-messaging",
        permanent: false,
        destination: "/learn/in-depth-tech/multi-chain-settlement/l1-to-l2-messaging",
      },
      {
        source: "/docs/dive-into-rooch/l2-to-l1-messaging",
        permanent: false,
        destination: "/learn/in-depth-tech/multi-chain-settlement/l2-to-l1-messaging",
      },
      {
        source: "/docs/tech-highlights/multi-chain-settlement",
        permanent: false,
        destination: "/learn/in-depth-tech/multi-chain-settlement/multi-chain-settlement",
      },
      {
        source: "/docs/tech-highlights/pipeline-transaction-processing",
        permanent: false,
        destination: "/learn/in-depth-tech/pipeline-transaction-processing",
      },
      {
        source: "/docs/tech-highlights/sequence-proof",
        permanent: false,
        destination: "/learn/in-depth-tech/sequence-proof",
      },
      {
        source: "/docs/tech-highlights/storage",
        permanent: false,
        destination: "/learn/core-concepts/objects/storage",
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
        source: "/docs/developer-guides/installation",
        permanent: false,
        destination: "/build/getting-started/installation",
      },
      {
        source: "/docs/developer-guides/run-local-testnet",
        permanent: false,
        destination: "/build/getting-started/connect-to-rooch/run-local-testnet",
      },
      {
        source: "/docs/developer-guides/connect-devnet",
        permanent: false,
        destination: "/build/getting-started/connect-to-rooch/connect-devnet",
      },
      {
        source: "/docs/developer-guides/connect-testnet",
        permanent: false,
        destination: "/build/getting-started/connect-to-rooch/connect-testnet",
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
      {
        source: "/developer-guides/hash",
        permanent: false,
        destination: "/build/rooch-framework/cryptographic-primitives/hash",
      },
      {
        source: "/docs/developer-guides/timestamp",
        permanent: false,
        destination: "/build/rooch-framework/timestamp",
      },
      {
        source: "/docs/developer-guides/private-generics",
        permanent: false,
        destination: "/build/rooch-framework/private-generics",
      },
      {
        source: "/docs/developer-guides/unit-test",
        permanent: false,
        destination: "/build/rooch-framework/unit-test",
      },
      {
        source: "/docs/example-guides",
        permanent: false,
        destination: "/build/example-guides",
      },
      {
        source: "/docs/developer-guides/cli",
        permanent: false,
        destination: "/build/reference/rooch-cli",
      },
      {
        source: "/docs/developer-guides/typescript-sdk",
        permanent: false,
        destination: "/build/reference/sdk/typescript-sdk",
      },
      {
        source: "/docs/developer-guides/access-path",
        permanent: false,
        destination: "/build/reference/rpc/access-path",
      },
    ];
  },
});
