// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

///<reference types="vite/client" />

interface ImportMetaEnv extends Readonly<Record<string, string>> {
  readonly VITE_ROOCH_OPERATING_ADDRESS: string
  // more env variables...
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
