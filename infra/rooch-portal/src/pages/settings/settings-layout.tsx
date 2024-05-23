// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import ManageSessions from '@/pages/settings/components/manage-sessions'
import { RoochAddress } from '@/pages/settings/components/rooch-address'

export const SettingsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Connected Account section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">Rooch Address</h1>
            <p className="text-muted-foreground">
              Your Rooch address is used in the smart contract functions.
            </p>
          </span>
        </div>
        <RoochAddress />
      </div>
      {/* Manage Sessions section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">Manage Sessions</h1>
            <p className="text-muted-foreground">Account is connected to these sites.</p>
          </span>
        </div>
        <ManageSessions />
      </div>
    </div>
  )
}
