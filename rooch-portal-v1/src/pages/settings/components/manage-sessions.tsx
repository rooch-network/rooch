import React from 'react'

import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

import { NoData } from '@/components/no-data'

interface App {
  app: string
  appSite: string
  contract: string
  sessionID: number
  grantedAt: string
  expiredAt: string
}

const apps: App[] = [
  {
    app: '',
    appSite: '',
    contract: '',
    sessionID: 0,
    grantedAt: '',
    expiredAt: '',
  },
]

export const ManageSessions: React.FC = () => {
  const hasValidData = (apps: App[]): boolean => {
    return apps.some(
      (app) =>
        app.app ||
        app.appSite ||
        app.contract ||
        app.sessionID !== 0 ||
        app.grantedAt ||
        app.expiredAt,
    )
  }

  if (!hasValidData(apps)) {
    return <NoData />
  }

  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Manage the connected apps.</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">App</TableHead>
            <TableHead>Session Key ID</TableHead>
            <TableHead>Session ID</TableHead>
            <TableHead>Granted at</TableHead>
            <TableHead>Expired at</TableHead>
            <TableHead className="text-center">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {apps.map((app, index) => (
            <TableRow key={index}>
              <TableCell className="font-medium">{app.app}</TableCell>
              <TableCell>{app.contract}</TableCell>
              <TableCell>{app.sessionID}</TableCell>
              <TableCell>{app.grantedAt}</TableCell>
              <TableCell>{app.expiredAt}</TableCell>
              <TableCell className="text-center">
                <button className="text-red-500">Disconnect</button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

export default ManageSessions
