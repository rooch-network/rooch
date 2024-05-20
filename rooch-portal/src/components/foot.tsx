// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Button } from './ui/button'

export const Foot = () => {
  const handleOnClick = () => {
    window.open('https://rooch.network/', '_blank')
  }

  return (
    <div className="flex items-center justify-start bg-background/95">
      <Button
        onClick={handleOnClick}
        variant="link"
        size="sm"
        className="text-muted-foreground/60 p-0 ml-6"
      >
        <h3>© Root Branch Ltd. </h3>
      </Button>
      <p className="text-muted-foreground text-sm ml-1">2024. All rights reserved.</p>
    </div>
  )
}
