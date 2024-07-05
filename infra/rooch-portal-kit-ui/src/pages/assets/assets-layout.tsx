// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { AssetsDetails } from './assets-details/assets-details'
import { ProfileCard } from './profile-card/profile-card'

export const AssetsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      <ProfileCard />
      <AssetsDetails />
    </div>
  )
}
