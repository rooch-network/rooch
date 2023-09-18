// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { ReactNode, useState } from 'react'
import {
  Button,
  TextField,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
} from '@mui/material'

// ** Next Import
import { useRouter } from 'next/router'

// ** Hooks Import
import useSessionAccount from 'src/hooks/useSessionAccount'

interface Props {
  open: boolean
  onReqAuthorize: (scope: Array<string>, maxInactiveInterval: number) => void
  onLogout: () => void
}

const AuthDialog: React.FC<Props> = ({ open, onReqAuthorize, onLogout }) => {
  const [scope, setScope] = useState<Array<string>>(['0x1::*::*', '0x3::*::*'])
  const [maxInactiveInterval, setMaxInactiveInterval] = useState<number>(1200)

  const handleAuth = () => {
    onReqAuthorize && onReqAuthorize(scope, maxInactiveInterval)
  }

  return (
    <Dialog open={open} onClose={onLogout}>
      <DialogTitle>Session Authorize</DialogTitle>
      <DialogContent>
        <DialogContentText>
          The current session does not exist or has expired. Please authorize the creation of a new
          session.
        </DialogContentText>
        <TextField
          autoFocus
          margin="dense"
          id="scope"
          label="Scope"
          type="text"
          multiline
          fullWidth
          disabled
          variant="standard"
          value={scope.join('\n')}
          onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
            setScope(event.target.value.split('\n'))
          }}
        />
        <TextField
          autoFocus
          margin="dense"
          id="max_inactive_interval"
          label="Max Inactive Interval"
          type="text"
          multiline
          fullWidth
          disabled
          variant="standard"
          value={maxInactiveInterval}
          onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
            setMaxInactiveInterval(parseInt(event.target.value))
          }}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={onLogout}>Logout</Button>
        <Button onClick={handleAuth}>Authorize</Button>
      </DialogActions>
    </Dialog>
  )
}

interface SessionGuardProps {
  children: ReactNode
}

const SessionGuard = (props: SessionGuardProps) => {
  const { children } = props
  const router = useRouter()
  const { sessionAccount, requestAuthorize } = useSessionAccount()

  const handleAuth = (scope: Array<string>, maxInactiveInterval: number) => {
    requestAuthorize(scope, maxInactiveInterval)
  }

  const hanleLogout = () => {
    if (router.asPath !== '/') {
      router.replace({
        pathname: '/login',
        query: { returnUrl: router.asPath },
      })
    } else {
      router.replace('/login')
    }
  }

  const isSessionInvalid = () => {
    return sessionAccount === undefined
  }

  return (
    <div>
      <div>
        <AuthDialog
          open={isSessionInvalid()}
          onReqAuthorize={handleAuth}
          onLogout={hanleLogout}
        ></AuthDialog>
      </div>
      <div>{children}</div>
    </div>
  )
}

export default SessionGuard
