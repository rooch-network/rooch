// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode } from 'react'
import * as styles from './Card.css.js'
import { Text } from './Text.js'
import { Heading } from './Heading.js'

type CardProps = {
  header?: string
  headerRight?: string
  children: ReactNode
  footer?: ReactNode
}

export function Card({ header, headerRight, children, footer }: CardProps) {
  return (
    <div className={styles.card}>
      {(header || headerRight) && (
        <div className={styles.cardHeader}>
          <Heading as="h3" size="sm" weight="normal">
            {header}
          </Heading>
          {headerRight && (
            <Text weight="medium" color="muted">
              {headerRight}
            </Text>
          )}
        </div>
      )}
      <div className={styles.cardBody}>{children}</div>
      {footer && <div className={styles.cardFooter}>{footer}</div>}
    </div>
  )
}
