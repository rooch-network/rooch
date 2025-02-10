interface ImportMetaEnv {
  readonly VITE_ROOCH_RPC_URL: string
  readonly VITE_PACKAGE_ID: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}