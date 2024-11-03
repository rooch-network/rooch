'use client';

import Typography from '@mui/material/Typography';
import { Box, Card, Stack, Button, CardHeader, CardContent } from '@mui/material';

import { CONFIG } from 'src/config-global';
import { DashboardContent } from 'src/layouts/dashboard';

export interface AppItemProps {
  id: number;
  name: string;
  description: string;
  profileUrl: string;
  logoUrl: string;
  type: string;
  url: string;
}

const AppList: AppItemProps[] = [
  {
    id: 1,
    name: 'Rooch Clicker',
    description: "Join our Click Challenge!",
    logoUrl: `${CONFIG.site.basePath}/logo/logo-single.svg`,
    profileUrl: `${CONFIG.site.basePath}/assets/apps/clicker-app.png`,
    type: 'Demo',
    url: 'https://clicker.rooch.io',
  },
  {
    id: 2,
    name: 'Grow Bitcoin',
    description: "Backing Ideas with Bitcoin Staking!",
    logoUrl: `${CONFIG.site.basePath}/logo/logo-single.svg`,
    profileUrl: `${CONFIG.site.basePath}/assets/apps/grow-app.png`,
    type: 'Demo',
    url: 'https://test-grow.rooch.network',
  },
];

export default function AppsView() {
  return (
    <DashboardContent maxWidth="xl">
      <Stack flexDirection="column" justifyContent="space-between">
        <Typography variant="h4">Apps</Typography>
        <Typography className="text-gray-400 font-normal text-base">
          Explore Bitcoin applications powered by Rooch
        </Typography>
      </Stack>

      <Card className="mt-4">
        <CardContent
          component={Box}
          gap={3}
          display="grid"
          gridTemplateColumns={{
            xs: 'repeat(1, 1fr)',
            sm: 'repeat(2, 1fr)',
            md: 'repeat(3, 1fr)',
            lg: 'repeat(4, 1fr)',
          }}
        >
          {AppList.map((i) => (
            <Card key={i.id} elevation={0} className="!bg-gray-100 !shadow-none">
              <CardHeader
                title={
                  <Stack direction="row" alignItems="center">
                    <img src={i.logoUrl} className="w-8 h-8" alt="logo" />
                    {i.name}
                  </Stack>
                }
                titleTypographyProps={{
                  sx: {
                    fontSize: '1rem !important',
                  },
                }}
                subheader={i.description}
              />
              <CardContent className="!pt-2 rounded">
                <img className="rounded-xl" src={`${i.profileUrl}`} width="100%" alt="clicker" />
                <a
                  href={i.url}
                  className="w-full text-black"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Button variant="outlined" className="!mt-4" fullWidth>
                    Go to App
                  </Button>
                </a>
              </CardContent>
            </Card>
          ))}
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
