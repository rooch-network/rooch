// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as styles from './ConnectView.css.js'
import { Button } from '../../../components/ui/Button.js'
import { Heading } from '../../../components/ui/Heading.js'
import { useEffect, useMemo, useState } from 'react'
import { LocalAccountType, LocalActiveAddress, LocalWallet } from '../../../wallet/local.js'
import { BitcoinNetowkType, Secp256k1Keypair, toShortStr } from '@roochnetwork/rooch-sdk'
// import { useRoochClient } from '../../../hooks/index.js'
import { PlusIcon } from '../../../components/icons/PlusIcon.js'
import { TrashIcon } from '../../../components/icons/TrashIcon.js'
import { useCurrentNetwork } from '../../../hooks/index.js'

type LocalWalletConnectViewProps = {
  onConnect: () => void
}

export function LocalWalletConnectView({ onConnect }: LocalWalletConnectViewProps) {
  const [accounts, setAccounts] = useState<Map<string, LocalAccountType>>(new Map())
  // const [accountInfo, setAccountInfo] = useState<Map<string, BalanceInfoView>>(new Map())
  const [showImportForm, setShowImportForm] = useState(false)
  const [importValue, setImportValue] = useState('')
  const [copiedAddress, setCopiedAddress] = useState<string | null>(null)
  // const client = useRoochClient()
  const network = useCurrentNetwork()

  useEffect(() => {
    const accounts = LocalWallet.getAccounts()
    setAccounts(accounts)
  }, [])

  const addresss = useMemo(() => {
    const addresses = new Map<string, any[]>()
    for (const account of accounts.entries()) {
      const addressList = []
      for (const key of account[1].keys) {
        const address = Secp256k1Keypair.fromSecretKey(key)
          .getBitcoinAddressWith(
            network === 'mainnet' ? BitcoinNetowkType.Bitcoin : BitcoinNetowkType.Testnet,
          )
          .toStr()
        addressList.push({
          address,
          key,
        })
      }
      addresses.set(account[0], addressList)
    }
    return addresses
  }, [accounts, network])

  // useEffect(() => {
  //   if (accountInfo.size !== 0) return
  //   const info = new Map<string, BalanceInfoView>()
  //   const fetchBalance = async () => {
  //     for (const account of accounts.entries()) {
  //       for (const address of account[1].address) {
  //         const balance = await client.getBalance({
  //           owner: address.address,
  //           coinType: '0x3::gas_coin::RGas',
  //         })
  //         info.set(address.address, balance)
  //       }
  //     }
  //   }

  //   setAccountInfo(info)

  //   fetchBalance()
  // }, [accountInfo.size, accounts, client])

  const handleCreateAccount = async () => {
    const accounts = LocalWallet.createAccount()
    setAccounts(accounts)
  }

  const handleCreateAddress = async (accountName: string) => {
    const accounts = LocalWallet.createAddress(accountName)
    setAccounts(accounts)
  }

  const handleImport = async () => {
    const accounts = LocalWallet.importAccount(importValue)
    setAccounts(accounts)
    setShowImportForm(false)
  }

  const handleDeleteAddress = (accountName: string, address: string) => {
    const accounts = LocalWallet.removeAddress(accountName, address)
    setAccounts(accounts)
  }

  const handleDeleteAccount = (accountName: string) => {
    const accounts = LocalWallet.removeAccount(accountName)
    setAccounts(accounts)
  }

  const handleConnect = (key: string) => {
    window.localStorage.setItem(LocalActiveAddress, key)
    onConnect()
  }

  const handleCopy = async (address: string) => {
    try {
      await navigator.clipboard.writeText(address)
      setCopiedAddress(address)
      setTimeout(() => setCopiedAddress(null), 2000)
    } catch (err) {
      console.error('Failed to copy address:', err)
    }
  }

  console.log(accounts)

  return (
    <div className={styles.container}>
      <Heading size="lg" className={styles.title}>
        Local Wallet
      </Heading>

      {!showImportForm && (
        <>
          <div className={styles.addressList}>
            {Array.from(addresss.entries()).map(([accountName, account]) => (
              <div key={accountName} className={styles.accountSection}>
                <div className={styles.accountHeader}>
                  <span className={styles.accountName}>{accountName}</span>
                  <div className={styles.accountActions}>
                    <Button
                      variant="outline"
                      className={styles.iconButton}
                      onClick={() => handleCreateAddress(accountName)}
                      title="Derive New Address"
                    >
                      <PlusIcon className={styles.icon} />
                    </Button>
                    <Button
                      variant="outline"
                      className={styles.iconButton}
                      onClick={() => handleDeleteAccount(accountName)}
                      title="Delete Account"
                    >
                      <TrashIcon className={styles.icon} />
                    </Button>
                  </div>
                </div>
                <div className={styles.addressList}>
                  {account.map((addr) => (
                    <div key={addr.address} className={styles.addressItem}>
                      <div className={styles.addressContent}>
                        <span
                          className={styles.addressText}
                          onClick={() => handleCopy(addr.address)}
                          style={{ cursor: 'pointer' }}
                        >
                          {toShortStr(addr.address, { start: 8, end: 4 })}
                        </span>
                        {copiedAddress === addr.address && (
                          <span className={styles.copyFeedback}>Copied!</span>
                        )}
                      </div>
                      {/* {copiedAddress !== addr.address && (
                        <span>
                          {accountInfo.get(addr.address)?.fixedBalance.toFixed(2) || '-'} Rgas
                        </span>
                      )} */}
                      <div className={styles.addressActions}>
                        <Button
                          variant="outline"
                          className={styles.actionButton}
                          onClick={() => handleConnect(addr.key)}
                        >
                          Connect
                        </Button>
                        <Button
                          variant="outline"
                          className={styles.actionButton}
                          onClick={() => handleDeleteAddress(accountName, addr.key)}
                        >
                          Delete
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
          <div className={styles.actions}>
            <Button variant="outline" onClick={handleCreateAccount}>
              Create
            </Button>
            <Button variant="outline" onClick={() => setShowImportForm(true)}>
              Import
            </Button>
          </div>
        </>
      )}

      {showImportForm && (
        <div className={styles.importForm}>
          <textarea
            className={styles.importInput}
            value={importValue}
            onChange={(e) => setImportValue(e.target.value)}
            placeholder={`Enter your mnemonic`}
          />
          <div className={styles.importActions}>
            <Button variant="outline" onClick={() => setShowImportForm(false)}>
              Cancel
            </Button>
            <Button variant="outline" onClick={handleImport}>
              Import
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
