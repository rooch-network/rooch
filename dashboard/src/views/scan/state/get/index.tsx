// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState } from 'react'

// ** MUI Imports
import Grid from '@mui/material/Grid'
import Card from '@mui/material/Card'
import TextField from '@mui/material/TextField'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'

// ** SDK Imports
import CardSnippet from 'src/@core/components/card-snippet'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'
import Spinner from '../../../../@core/components/spinner'

const StateGetView = () => {
  // ** State
  const [accessPath, setAccessPath] = useState<string>('/object/0x1')

  let { data, isPending, error } = useRoochClientQuery('getStates', accessPath, {
    refetchOnWindowFocus: false,
    retry: false,
    enabled: !!accessPath,
  })

  return (
    <Grid container spacing={6}>
      <Grid item xs={12}>
        <Card>
          <CardHeader title="State Filters" />
          <CardContent>
            <TextField
              id="access-path-id"
              label="Access Path"
              fullWidth
              value={accessPath}
              helperText={error?.toString()}
              onChange={(v) => setAccessPath(v.target.value)}
            />
          </CardContent>
        </Card>
      </Grid>

      <Grid item xs={12}>
        {error !== null ? null : isPending ? (
          <Spinner />
        ) : (
          <CardSnippet
            defaultShow={true}
            fullHeight={true}
            codes={[
              {
                code: JSON.stringify(data, null, 2),
                lng: 'json',
              },
            ]}
          />
        )}
      </Grid>
    </Grid>
  )
}

export default StateGetView
