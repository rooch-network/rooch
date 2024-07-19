// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export const Loading = (loading: boolean) => {
  return (
    loading ?
      <div className="relative p-24">
        <div className="absolute inset-0 bg-inherit bg-opacity-50 flex justify-center items-center">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
        </div>
      </div> :
      <></>
  )
}
