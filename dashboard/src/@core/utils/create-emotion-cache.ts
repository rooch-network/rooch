// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import createCache from '@emotion/cache'

export const createEmotionCache = () => {
  return createCache({ key: 'css' })
}
