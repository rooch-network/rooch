// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Heading } from '../../ui/Heading.js'
import * as styles from './WhatIsAWallet.css.js'
import { Text } from '../../ui/Text.js'

export function WhatIsAWallet() {
  return (
    <div className={styles.container}>
      <Heading as="h2">What is a Session</Heading>
      <div className={styles.content}>
        <Text weight="medium" color="muted">
          Rooch's Session Key is a temporary key that facilitates users to interact with the chain.
        </Text>
        <Text>
          When interacting with Rooch applications, each application generates a session key. It has
          an expiration time and will become invalid if there is no interaction for a long time.
        </Text>
      </div>
    </div>
  )
}
