'use client';

import { useState } from 'react';

import { Card, Stack, Button, TextField, CardHeader, CardContent } from '@mui/material';

import { RouterLink } from 'src/routes/components';

import { DashboardContent } from 'src/layouts/dashboard';

const placeholder = 'tb1pjugffa0n2ts0vra032t3phae7xrehdjfzkg284ymvf260vjh225s5u4z76';

export default function SearchView() {
  const [account, setAccount] = useState('');

  return (
    <DashboardContent maxWidth="xl">
      <Card>
        <CardHeader
          title="Search Account"
          subheader="Enter Bitcoin Address to search"
          sx={{ mb: 2 }}
        />
        <CardContent className="!pt-0">
          <Stack direction="row" alignItems="center" className="w-full" spacing={2}>
            <TextField
              size="small"
              className="w-full"
              value={account}
              placeholder={placeholder}
              onChange={(e) => {
                setAccount(e.target.value);
              }}
            />
            <Button component={RouterLink} href={`/account/${account || placeholder}`}>
              Search
            </Button>
          </Stack>
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
