// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export function KeyIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M10.6667 5.33334C10.6667 6.80609 9.47343 8 8.00067 8C6.52792 8 5.33334 6.80609 5.33334 5.33334C5.33334 3.86059 6.52792 2.66667 8.00067 2.66667C9.47343 2.66667 10.6667 3.86059 10.6667 5.33334Z"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M8 8V14.6667"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M6 12H10"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}
