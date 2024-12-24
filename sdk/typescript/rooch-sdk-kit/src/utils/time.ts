// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import dayjs from 'dayjs'

export function getUTCOffset() {
  const date = new Date()
  const offset = -date.getTimezoneOffset()
  const hours = Math.floor(offset / 60)
  const minutes = offset % 60
  return `UTC ${hours >= 0 ? '+' : '-'}${hours}:${minutes < 10 ? '0' : ''}${minutes}`
}

export const unix2str = (input: number) => {
  const timestampInSeconds = input > 1000000000000 ? input / 1000 : input

  return `${dayjs.unix(timestampInSeconds).format('MMM DD, YYYY HH:mm:ss')}`
}

export const second2Countdown = (input: number) => {
  const days = Math.floor(input / (24 * 3600))
  const hours = Math.floor((input % (24 * 3600)) / 3600)
  const minutes = Math.floor((input % 3600) / 60)
  const secs = Math.floor(input % 60)

  return `${days} : ${hours} : ${minutes} : ${secs}`
}
