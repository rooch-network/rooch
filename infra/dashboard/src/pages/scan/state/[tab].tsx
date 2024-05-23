// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, ReactElement, useEffect } from 'react'

// ** MUI Imports
import Grid from '@mui/material/Grid'
import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import { useRouter } from 'next/router'
import useMediaQuery from '@mui/material/useMediaQuery'
import { styled } from '@mui/material/styles'
import CircularProgress from '@mui/material/CircularProgress'
import Tab from '@mui/material/Tab'
import MuiTabList from '@mui/lab/TabList'
import TabPanel from '@mui/lab/TabPanel'
import StateGetView from 'src/views/scan/state/get'
import StateListView from 'src/views/scan/state/list'
import { TabContext } from '@mui/lab'

// ** Icon
import Icon from 'src/@core/components/icon'

enum Actions {
  Get = 'get',
  List = 'list',
}

const TabList = styled(MuiTabList)(({ theme }) => ({
  minHeight: 40,
  marginBottom: theme.spacing(4),
  '& .MuiTabs-indicator': {
    display: 'none',
  },
  '& .MuiTab-root': {
    minWidth: 65,
    minHeight: 40,
    paddingTop: theme.spacing(2.5),
    paddingBottom: theme.spacing(2.5),
    borderRadius: theme.shape.borderRadius,
    '&.Mui-selected': {
      color: theme.palette.common.white,
      backgroundColor: theme.palette.primary.main,
    },
    [theme.breakpoints.up('sm')]: {
      minWidth: 130,
    },
  },
}))

const StateList = () => {
  // ** Hooks
  const router = useRouter()
  const hideText = useMediaQuery((theme: any) => theme.breakpoints.down('sm'))

  // ** State
  const [activeTab, setActiveTab] = useState(router.query.tab as Actions)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    if (router.query.tab) {
      setActiveTab(router.query.tab as Actions)
      setIsLoading(false)
    }
  }, [router.query])

  const handleChange = (event: any, value: any) => {
    setIsLoading(true)
    setActiveTab(value)
    router
      .push({
        pathname: `/scan/state/${value.toLowerCase()}`,
      })
      .then(() => setIsLoading(false))
  }

  const tabContentList: { [key in Actions]: ReactElement } = {
    [Actions.Get]: <StateGetView />,
    [Actions.List]: <StateListView />,
  }

  return !activeTab ? (
    <></>
  ) : (
    <TabContext value={activeTab}>
      <Grid>
        <Grid item xs={12}>
          <TabList variant="scrollable" scrollButtons="auto" onChange={handleChange}>
            <Tab
              value="get"
              label={
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    ...(!hideText && { '& svg': { mr: 2 } }),
                  }}
                >
                  <Icon icon="bxs-search" />
                  {!hideText && 'Get'}
                </Box>
              }
            />
            <Tab
              value="list"
              label={
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    ...(!hideText && { '& svg': { mr: 2 } }),
                  }}
                >
                  <Icon icon="bx-list-ul" />
                  {!hideText && 'List'}
                </Box>
              }
            />
          </TabList>
        </Grid>
      </Grid>
      {isLoading ? (
        <Box sx={{ mt: 6, display: 'flex', alignItems: 'center', flexDirection: 'column' }}>
          <CircularProgress sx={{ mb: 4 }} />
          <Typography>Loading...</Typography>
        </Box>
      ) : (
        <TabPanel
          sx={{ p: 0, border: 0, boxShadow: 0, backgroundColor: 'transparent' }}
          value={activeTab}
        >
          {tabContentList[activeTab]}
        </TabPanel>
      )}
    </TabContext>
  )
}

export default StateList
