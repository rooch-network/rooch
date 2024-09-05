/**
 * @type {import('next').NextConfig}
 */

const isStaticExport = 'false';

const isProduction = process.env.NODE_ENV === 'production';

const iconDomains = ['https://api.unisvg.com', 'https://api.iconify.design', 'https://api.simplesvg.com'];
const apiDomains = ['https://dev-seed.rooch.network', 'https://test-seed.rooch.network'];

const cspHeader = `
    default-src 'self';
    script-src 'self' ${isProduction ? '' : "'unsafe-eval' 'unsafe-inline'"};
    style-src 'self' 'unsafe-inline';
    img-src 'self' blob: data: ${iconDomains.join(' ')};
    font-src 'self';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
    connect-src 'self' ${apiDomains.join(' ')} ${iconDomains.join(' ')};
`;

const nextConfig = {
  compiler: {
    removeConsole: true,
  },
  trailingSlash: false,
  basePath: process.env.NEXT_PUBLIC_BASE_PATH,
  env: {
    BUILD_STATIC_EXPORT: isStaticExport,
  },
  modularizeImports: {
    '@mui/icons-material': {
      transform: '@mui/icons-material/{{member}}',
    },
    '@mui/material': {
      transform: '@mui/material/{{member}}',
    },
    '@mui/lab': {
      transform: '@mui/lab/{{member}}',
    },
  },
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'Content-Security-Policy',
            value: cspHeader.replace(/\n/g, ''),
          },
        ],
      },
    ];
  },
  webpack(config) {
    config.module.rules.push({
      test: /\.svg$/,
      use: ['@svgr/webpack'],
    });

    return config;
  },
  ...(isStaticExport === 'true' && {
    output: 'export',
  }),
};

export default nextConfig;
