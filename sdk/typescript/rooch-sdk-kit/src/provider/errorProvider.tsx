// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createContext, ReactNode, useContext, useMemo, useState } from 'react'

export type ErrorType = {
  code: number
  msg: string
}

export interface ErrorProviderContext {
  error: ErrorType | null
  setError: (error: ErrorType) => void
  resolve: () => void
}

const ErrorContext = createContext<ErrorProviderContext | null>(null)

export const useError = () => {
  const ctx = useContext(ErrorContext)!
  return {
    error: ctx.error,
    resolve: ctx.resolve,
  }
}

export const useSetError = () => {
  const ctx = useContext(ErrorContext)!
  return ctx.setError
}

export type ErrorProviderProps = {
  children: ReactNode
}

export const ErrorProvider = (props: ErrorProviderProps) => {
  const { children } = props
  const [error, setError] = useState<ErrorType | null>(null)

  const ctx = useMemo((): ErrorProviderContext => {
    return {
      error: error,
      setError: setError,
      resolve: () => {
        setError(null)
      },
    }
  }, [error])
  return <ErrorContext.Provider value={ctx}>{children}</ErrorContext.Provider>
}
