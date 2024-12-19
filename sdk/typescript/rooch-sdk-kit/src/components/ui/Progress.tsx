// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as styles from '../fauct-modal/views/FaucetView.css.js'
import { useProgress } from '../ProgressProvider.js'

export function Progress() {
  const { progress } = useProgress()
  return <div className={styles.progressBar} style={{ width: `${progress}%` }} />
}
