// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Slot } from '@radix-ui/react-slot'
import clsx from 'clsx'
import type { InputHTMLAttributes } from 'react'
import { forwardRef } from 'react'

import { inputVariants } from './Input.css.js'
import type { InputVariants } from './Input.css.js'

type InputProps = {
  asChild?: boolean
} & InputHTMLAttributes<HTMLInputElement> &
  InputVariants

const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, variant, size, asChild = false, ...props }, forwardedRef) => {
    const Comp = asChild ? Slot : 'input'
    return (
      <Comp
        {...props}
        className={clsx(inputVariants({ variant, size }), className)}
        ref={forwardedRef}
      />
    )
  },
)
Input.displayName = 'Button'

export { Input }
