// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { createContext, ReactNode, useCallback, useContext, useState } from 'react'
import { Progress } from './ui/Progress.js'

export interface ProgressProviderContext {
  progress: number
  loading: boolean
  start: () => void
  finish: (callback?: () => void) => void
}

const ProgressProviderContext = createContext<ProgressProviderContext | null>(null)

export const ProgressProvider = ({ children }: { children: ReactNode }) => {
  const [progress, setProgress] = useState(0)
  const [loading, setLoading] = useState(false)

  const start = useCallback(() => {
    setLoading(true)
    setProgress(0)
    const interval = setInterval(() => {
      setProgress((prev) => {
        const nextProgress = prev + 10
        if (nextProgress >= 70) {
          clearInterval(interval)
        }
        return Math.min(nextProgress, 70)
      })
    }, 100)
  }, [])

  const finish = useCallback((callback?: () => void) => {
    const interval = setInterval(() => {
      setProgress((prev) => {
        const nextProgress = prev + 5
        if (nextProgress >= 100) {
          clearInterval(interval)
          setTimeout(() => {
            setLoading(false)
            if (callback) {
              callback()
            }
          }, 300)
        }
        return Math.min(nextProgress, 100)
      })
    }, 50)
  }, [])

  return (
    <ProgressProviderContext.Provider value={{ loading, progress, start, finish }}>
      {children}
      {loading && <Progress />}
    </ProgressProviderContext.Provider>
  )
}

export const useProgress = () => {
  const ctx = useContext(ProgressProviderContext)
  if (!ctx) {
    throw new Error('useSubscribeToError must be used within a GlobalProvider')
  }
  return ctx
}
