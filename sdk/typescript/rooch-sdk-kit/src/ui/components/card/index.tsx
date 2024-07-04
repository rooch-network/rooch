// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import * as React from 'react'

const cardStyle: React.CSSProperties = {
  borderRadius: '1rem', // rounded-xl
  border: '1px solid', // border
  backgroundColor: '#27272a', // bg-card
  color: '#fafafa', // text-card-foreground
  boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06)', // shadow
}
const cardHeaderStyle: React.CSSProperties = {
  display: 'flex', // flex
  flexDirection: 'column', // flex-col
  gap: '0.375rem', // space-y-1.5 -> 1.5 * 0.25rem = 0.375rem
  padding: '1.5rem', // p-6 -> 6 * 0.25rem = 1.5rem
}
const cardTitleStyle: React.CSSProperties = {
  fontWeight: 600, // font-semibold
  lineHeight: '1', // leading-none
  letterSpacing: '-0.01562em', // tracking-tight
}
const cardDescriptionStyle: React.CSSProperties = {
  fontSize: '0.875rem', // text-sm
  color: '#a1a1aa',
}
const cardContentStyle: React.CSSProperties = {
  padding: '1.5rem', // p-6 -> 6 * 0.25rem = 1.5rem
  paddingTop: '0',
}
const cardFooterStyle: React.CSSProperties = {
  display: 'flex', // flex
  alignItems: 'center', // items-center
  padding: '1.5rem', // p-6 -> 6 * 0.25rem = 1.5rem
  paddingTop: '0', // pt-0
}
export const Card = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      style={cardStyle}
      // className={cn('rounded-xl border bg-card text-card-foreground shadow', className)}
      {...props}
    />
  ),
)
Card.displayName = 'Card'

export const CardHeader = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      style={cardHeaderStyle}
      // className={cn('flex flex-col space-y-1.5 p-6', className)}
      {...props}
    />
  ),
)
CardHeader.displayName = 'CardHeader'

export const CardTitle = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  // eslint-disable-next-line jsx-a11y/heading-has-content
  <h3
    ref={ref}
    style={cardTitleStyle}
    // className={cn('font-semibold leading-none tracking-tight', className)}
    {...props}
  />
))
CardTitle.displayName = 'CardTitle'

export const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    style={cardDescriptionStyle}
    // className={cn('text-sm text-muted-foreground', className)}
    {...props}
  />
))
CardDescription.displayName = 'CardDescription'

export const CardContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      style={cardContentStyle}
      // className={cn('p-6 pt-0', className)}
      {...props}
    />
  ),
)
CardContent.displayName = 'CardContent'

export const CardFooter = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      style={cardFooterStyle}
      // className={cn('flex items-center p-6 pt-0', className)}
      {...props}
    />
  ),
)
CardFooter.displayName = 'CardFooter'
