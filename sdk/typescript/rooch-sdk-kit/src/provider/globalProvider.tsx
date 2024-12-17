// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { createContext, ReactNode, useContext } from 'react'

export type ErrorType = {
  code: number
  message: string
}

export type RequestStatusType = 'requesting' | 'success'
type RequestCallback = (status: RequestStatusType) => void

class RequestEventManager {
  private callbacks: RequestCallback[] = []

  subscribe(callback: RequestCallback) {
    this.callbacks.push(callback)
    return () => {
      this.callbacks = this.callbacks.filter((cb) => cb !== callback)
    }
  }

  trigger(status: RequestStatusType) {
    this.callbacks.forEach((callback) => callback(status))
  }
}

type ErrorCallback = (error: ErrorType) => void

class ErrorEventManager {
  private callbacks: ErrorCallback[] = []

  subscribe(callback: ErrorCallback) {
    this.callbacks.push(callback)
    return () => {
      this.callbacks = this.callbacks.filter((cb) => cb !== callback)
    }
  }

  trigger(error: ErrorType) {
    this.callbacks.forEach((callback) => callback(error))
  }
}

export interface GlobalProviderContext {
  triggerError: (error: ErrorType) => void
  subscribeOnError: (callback: (error: ErrorType) => void) => () => void
  triggerRequest: (status: RequestStatusType) => void
  subscribeOnRequest: (callback: (status: RequestStatusType) => void) => () => void
}

const GlobalContext = createContext<GlobalProviderContext | null>(null)

export const GlobalProvider = ({ children }: { children: ReactNode }) => {
  const errorEventManager = new ErrorEventManager()
  const requestEventManager = new RequestEventManager()
  const triggerError = (error: ErrorType) => {
    errorEventManager.trigger(error)
  }

  const subscribeOnError = (callback: (error: ErrorType) => void) => {
    return errorEventManager.subscribe(callback)
  }

  const triggerRequest = (status: RequestStatusType) => {
    requestEventManager.trigger(status)
  }

  const subscribeOnRequest = (callback: (status: RequestStatusType) => void) => {
    return requestEventManager.subscribe(callback)
  }

  return (
    <GlobalContext.Provider
      value={{ triggerError, subscribeOnError, triggerRequest, subscribeOnRequest }}
    >
      {children}
    </GlobalContext.Provider>
  )
}

export const useSubscribeOnError = () => {
  const ctx = useContext(GlobalContext)
  if (!ctx) {
    throw new Error('useSubscribeToError must be used within a GlobalProvider')
  }
  return ctx.subscribeOnError
}

export const useTriggerError = () => {
  const ctx = useContext(GlobalContext)
  if (!ctx) {
    throw new Error('useTriggerError must be used within a GlobalProvider')
  }
  return ctx.triggerError
}

export const useSubscribeOnRequest = () => {
  const ctx = useContext(GlobalContext)
  if (!ctx) {
    throw new Error('useSubscribeToError must be used within a GlobalProvider')
  }
  return ctx.subscribeOnRequest
}

export const useTriggerRequest = () => {
  const ctx = useContext(GlobalContext)
  if (!ctx) {
    throw new Error('useTriggerError must be used within a GlobalProvider')
  }
  return ctx.triggerRequest
}
