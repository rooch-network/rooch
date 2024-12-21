'use client'

import { useEffect } from 'react'
import { useSubscribeOnError } from '@roochnetwork/rooch-sdk-kit'

import { toast } from 'src/components/snackbar'

export function ErrorGuard() {
  const subscribeToError = useSubscribeOnError()

  useEffect(() => {
    const unsubscribe = subscribeToError((error) => {
      toast.error(error.message)
    })

    return () => {
      unsubscribe()
    }
  }, [subscribeToError])

  return <></>
}
