// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode } from 'react'

import * as styles from './View.css.js'

import { Heading } from './Heading.js'
import { Button } from './Button.js'
import { useProgress } from '../ProgressProvider.js'

export type ViewProps = {
  title: string
  disabledAction?: boolean
  actionText?: string
  actionOnClick?: () => void
  children: ReactNode
}

export function View({ title, disabledAction, actionText, actionOnClick, children }: ViewProps) {
  const { loading } = useProgress()
  return (
    <div className={styles.container}>
      <Heading as="h2" className={styles.title}>
        {title}
      </Heading>
      <div className={styles.content}>{children}</div>
      {actionText && (
        <div className={styles.actionButtonContainer}>
          <Button
            disabled={loading || disabledAction}
            type="button"
            variant="outline"
            onClick={actionOnClick}
          >
            {actionText}
          </Button>
        </div>
      )}
    </div>
  )
}
