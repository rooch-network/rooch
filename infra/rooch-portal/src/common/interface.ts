// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { LucideIcon } from 'lucide-react'

export interface WalletsMaterialProps {
  name: string
  icon: string
  description: string
  types: string[]
  link: string
}

export interface SidebarItemProps {
  icon: LucideIcon
  label: string
  href: string
  onClose?: () => void
}

export interface SidebarRoutesProps {
  onClose: () => void
}

export interface SidebarProps {
  onClose: () => void
}

export interface AppItemProps {
  id: number
  name: string
  description: string
  profileUrl: string
  logoUrl: string
  type: string
}

export interface SftsProps {
  id: number
  sftName: string
  distribution: string
  totalSupply: number
}

export interface UTXO {
  id: number
  amount: number
  isStaked: boolean
  isSelected: boolean
}

export interface MonthYear {
  year: number
  month: number
}

export interface Fns {
  cardType(cardNumber: string): string
  formatCardNumber(cardNumber: string): string
  validateCardNumber(cardNumber: string): boolean
  validateCardCVC(cvc: string, type?: string): boolean
  validateCardExpiry(monthYear: string, year?: string): boolean
  cardExpiryVal(monthYear: string | HTMLInputElement): MonthYear
}

export type PaymentTypes = {
  fns: Fns
  formatCardCVC(elem: HTMLInputElement): HTMLInputElement
  restrictNumeric(elem: HTMLInputElement): HTMLInputElement
  formatCardNumber(elem: HTMLInputElement): HTMLInputElement
  formatCardExpiry(elem: HTMLInputElement): HTMLInputElement
}

// ** Tab Type
export type TabItem = {
  id: string
  label: string
  available: boolean
}
