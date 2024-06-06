// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export async function copyToClipboard(text: string, setCopied: (value: boolean) => void) {
  try {
    await navigator.clipboard.writeText(text)
    setCopied(true)
    console.log('Copied to clipboard', text)
    setTimeout(() => setCopied(false), 2000)
  } catch (err) {
    console.error('Could not copy text', err)
  }
}
