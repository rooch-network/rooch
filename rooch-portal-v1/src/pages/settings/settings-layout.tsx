// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import ManageSessions from "@/pages/settings/components/manage-sessions.tsx";

export const SettingsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Mangage Sessions section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">Manage Sessions</h1>
            <p className="text-muted-foreground text-wrap">
              Account {} is connected to these sites. They can view your account address.
            </p>
          </span>
        </div>
        <ManageSessions />
      </div>
    </div>
  )
}
