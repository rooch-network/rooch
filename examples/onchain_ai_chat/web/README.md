# Onchain AI Chat Web Interface

A web interface for the Onchain AI Chat application built with React, TypeScript, and Rooch Network SDK.

## Features

- AI-powered chat rooms using onchain smart contracts
- Wallet integration for user authentication
- Real-time message updates
- Public and private chat rooms
- Message history persistence

## Prerequisites

- Node.js (v18 or later)
- npm or pnpm
- Rooch Network local node running

## Quick Start

1. Install dependencies:
```bash
pnpm install
```

2. Set up environment variables:
```bash
cp .env.example .env
```
Update `.env` with your configuration:
```plaintext
VITE_ROOCH_RPC_URL=http://localhost:50051
VITE_PACKAGE_ID=your_package_id_here
```

3. Start development server:
```bash
pnpm dev
```

## Project Structure

```
web/
├── src/
│   ├── components/    # Reusable UI components
│   ├── hooks/        # Custom React hooks
│   ├── pages/        # Page components
│   ├── types/        # TypeScript type definitions
│   └── utils/        # Utility functions
├── public/           # Static assets
└── ...config files
```

## Technology Stack

- React 18
- TypeScript
- Vite
- Tailwind CSS
- Rooch Network SDK
- React Router DOM
- Headless UI
- Heroicons

## Development

### ESLint Configuration

The project uses ESLint with TypeScript support. To enable type-aware lint rules:

```js
// eslint.config.js
import react from 'eslint-plugin-react'
import tseslint from 'typescript-eslint'

export default tseslint.config({
  languageOptions: {
    parserOptions: {
      project: ['./tsconfig.json'],
      tsconfigRootDir: import.meta.dirname,
    },
  },
  settings: { 
    react: { version: '18.3' } 
  },
  plugins: {
    react,
  },
  rules: {
    ...react.configs.recommended.rules,
    ...react.configs['jsx-runtime'].rules,
  },
})
```

### Available Scripts

- `pnpm dev` - Start development server
- `pnpm build` - Build for production
- `pnpm preview` - Preview production build
- `pnpm lint` - Run ESLint

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
