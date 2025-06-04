// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  Authenticator,
  BitcoinNetowkType,
  Bytes,
  Secp256k1Keypair,
  ThirdPartyAddress,
  Transaction,
} from '@roochnetwork/rooch-sdk'
import { BitcoinWallet } from '../wallet/index.js'
import { WalletNetworkType } from './types.js'

export const LocalKey = 'local-wallet-key'
export const LocalActiveAddress = 'local-wallet-active-address'

export type LocalAccountType = {
  mnemonic: string
  keys: string[]
}

export class LocalWallet extends BitcoinWallet {
  private keypair?: Secp256k1Keypair
  private onNetworkChange?: (network: string) => void
  private onAccountsChange?: (account: string[]) => void

  static getAccounts(): Map<string, LocalAccountType> {
    const keyString = window.localStorage.getItem(LocalKey)
    let accounts = new Map<string, LocalAccountType>()
    if (keyString) {
      try {
        const parsed = JSON.parse(keyString)
        accounts = new Map(parsed.map(([k, v]: [string, LocalAccountType]) => [k, v]))
      } catch (e) {
        console.error('Failed to parse accounts from localStorage:', e)
        window.localStorage.removeItem(LocalKey)
      }
    }
    return accounts
  }

  static removeAddress(accountName: string, key: string) {
    let accounts = LocalWallet.getAccounts()
    if (accounts.has(accountName)) {
      const account = accounts.get(accountName)
      if (account) {
        account.keys = account.keys.filter((a) => a !== key)
      }
    }
    window.localStorage.setItem(LocalKey, JSON.stringify(Array.from(accounts.entries())))
    return accounts
  }

  static removeAccount(accountName: string) {
    let accounts = LocalWallet.getAccounts()
    accounts.delete(accountName)
    window.localStorage.setItem(LocalKey, JSON.stringify(Array.from(accounts.entries())))
    return accounts
  }

  static createAddress(accountName: string) {
    const accounts = LocalWallet.getAccounts()
    const account = accounts.get(accountName)
    if (account) {
      const kp = Secp256k1Keypair.deriveKeypair(
        account.mnemonic,
        `m/86'/0'/0'/0/${account.keys.length + 1}`,
      )
      account.keys.push(kp.getSecretKey())
    }
    window.localStorage.setItem(LocalKey, JSON.stringify(Array.from(accounts.entries())))
    return accounts
  }

  static createAccount() {
    const { mnemonic, keypair } = Secp256k1Keypair.generateWithMnemonic()
    let accounts = LocalWallet.getAccounts()
    accounts.set(`Account${accounts.size + 1}`, {
      mnemonic,
      keys: [keypair.getSecretKey()],
    })
    window.localStorage.setItem(LocalKey, JSON.stringify(Array.from(accounts.entries())))
    return accounts
  }

  static importAccount(mnemonic: string) {
    const accounts = LocalWallet.getAccounts()
    const newAccount = Secp256k1Keypair.deriveKeypair(mnemonic)
    accounts.set(`Account${accounts.size + 1}`, {
      mnemonic,
      keys: [newAccount.getSecretKey()],
    })
    window.localStorage.setItem(LocalKey, JSON.stringify(Array.from(accounts.entries())))
    return accounts
  }

  getName(): string {
    return 'Local'
  }

  getIcon(_?: 'dark' | 'light'): string {
    return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAgAAAAIACAYAAAD0eNT6AAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAOxAAADsQBlSsOGwAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAACAASURBVHic7d15nFxlne/x73NObV3dnfSSdBISliBbZBEMJICOsqgIsgjSIQlLmNE7uIxc9epct9H2OqN33GYGx3Fw7h0HkEQmoCgIKsJlGAUCBJEhEEIiewjZ6b26qs5z/whLEgj0UlXPOef5vF8vX7wSknq+kk6dbz+/c54yAoAJsscsfocqdomtmGNUMvuoaooasVlFZscvCK1VzowoY/uU01qT03+qGHzf3HHV426TA/4yrgMASCZ7zMKT7Uj4RQ3oWA2ZwphfwEhqsdtMMbpeRn9pVi7bXPuUAPaEAgBg1Ozx571JQ+Ff2wHzPg2Y1pq9cGit2nWn6bTnm9uWPlmz1wWwRxQAAK/Lzj9/kiL7eTtgLlKvmSFbx8UysqY9ulZtBy02t/dU6rgS4D0KAIBXsVKgoxf9NztsLlVfMEeVBr9XtNgXzCR7prl36R0NXRfwCAUAwMvs/POP1Yi+ZHvNuzSsrNMwGVkzpfoFs3LZ153mAFKKAgB4bsdcP/NVO6DTazrXrwUjma7oB+b+pZe4jgKkDQUA8JA9obtFfbkv2EFdpD7tJRvjtwIjmanVfzW/X/ZB11GANInx33oAtWaPWXSBHdan1RcerrIC13lGzUhmevQVc9/SHtdRgLSgAAApZ+cvPtEOmy+p37zN+Vx/IkJZM8O+16y4+teuowBpQAEAUsietHhfbdfn7aDOVV/Q4TpPzRQ1aA4pTjU3/GDQdRQg6TKuAwCojRfn+p+3g3aJXWPiPdcfr0EV9czAtZJOcx0FSLoUvkMAfknsXH+8QlnTVT7C3HfNQ66jAEnGDgCQQPa4Re/UoPmy7Q/ebtcneK4/HlUZO5z5kaQjXUcBkowdACAh7DsW7q3+4Iupm+uPRyCZvcsHmDuvWec6CpBU7AAAMWYP7c6pKfcJO2QvsY+b/V/+eF3fRZKGMt+RdJbrKEBSUQCAGLLzFn3IDplLba85TNuNYbPu1WyfOcV1BiDJeFcBYsIeu3C+SuGXba9O1rDJuc6TBGamPd/cc/VS1zmAJGIHAHBo57m+fdrzuf442GH7KUkUAGAc2AEAGmznub56metPSFaR0TPN5onbh11HAZKGHQCgQez8xWfZIX3W9gbztF0B/bsGygo0fcYn9YT4yGBgjHgHAurIHrd4noZND3P9OmqL/hisWvom1zGApKEAADXG8/oNFkpmRnW6WbHseddRgCRhBADUwC7P6z8R7K+q60QeqUqqmC9L+qjrKECSsAMATMBLc331BvO8OIc/rlq1KVj9oy7XMYAkoQAAY2SPP/8YDUafs/3BezVkmlzngSQjmZnVt5gVyx50HQVICkYAwCjs8rz+k6ZDCl1Hws6spJJ6JJ3jOAmQGBQAYA+Y6yeLHQze4zoDkCSMAIDdMNdPLjMrep9ZsfQm1zmAJGAHANCuc337DHP9pLLD5rOSKADAKFAA4C173EUzVap8yQ7qHPuUmSLLXD/x+nSclQKz4wODAbwORgDwyi7n8Pcx108jM73yEbPyx//sOgcQd+wAwAucw+8PWwo+LokCALwB3gWRWnb+4rkq2S/wvL5nQmvNDLWZFVf3uo4CxBk7AEiVl+f6Q+Ys+6yZJus6ERquaowq9nOSPuc6ChBn7AAg8Zjr41Um22eCh6/e23UMIM4oAEisV57XN/NUNjyvj1cEkjkw2s/ctvRJ11GAuGIEgETZea7P8/rYo0jSC/pfkpa4jgLEFQUAsWeP6+5QKftFO2QWM9fHaNl+c6brDECcMQJALNm5f56VGfjki3P92arytYqxM7OCPzErrvyt6xxAHLEDgFixixYdKlu9UH19H7IPhJ10VEzISOWLkt7rOgYQRxQAOGe7u2cqCM6V0RLZ6CjJSK2RVAylQdfpkGS2LzjBdQYgrvj2Ck7Y7u4mBcHpMrpIO75De3UZfTqQfYLz+TExZqYWm3t+tMx1DiBu2AFAw9ienkCPrjpeNrhQ0mJJLa/7G6ZG0pOhuOkPE2GH9SlJFABgN+wAoO5enuvLLJE0fUy/9w8ZqZcvU0xAVpFpLjWZVctHXEcB4oQdANSFXbhwL1nbLaOLZKO3jrdrmmmRbC9jAExAWYGa85+U9LeuowBxwrdWqJlRzfXHqiLZFVk+3R0T0xatC1YtPcB1DCBOKACYkDHP9cezxupQ2sRJv5iAUDKzzDRz51UbXUcB4oIRAMZlx1zfdmv1IxdLwb71XMt0RbIUAExEVdJw9a8kfdx1FCAuKAAYtVfP9Ruk3Uo5SdzChQmwg2aBKADAyxgB4HXVZa4/nhzrQmk9uwCYACOZ6eXDzX3XPOQ6ChAH7ADgVRox1x8rMy2SpQBgIqykKOyRdK7jJEAsUADwsp3m+kukYD/XeXbRYqWilQbZtML42f6AzwUAXsS7qedemeubCyU713We18XRwKgBs1f1VHPvsl+6zgG4xg6Ah3ad69v3yiiTiPN2u6z0pBIRFfFly8HnJFEA4D0KgCd2mesbLZJVq+tMY5a30qRIeoF7ATABfTreSoHheCl4jhFAytnu7jcrDBdIdomk/VznmbANgexjjAEwMWZG9cPmvmWXu84BuMQOQArZCy6YoXJ5gYy6Jb0tVXvmUyJpXSBFdFeMny0FfyGJAgCv8S6aEvbiiwsaGjrD9fP6jWBXZ6RNfOliAjKyZnZpkrl9eb/rKIArqb1I+GCXuX5paJFMAuf642C6qrKb+NLFBFRk1J/7rKQvuo4CuMK3UQmUurn+WFnJ3pPlaGBMTJt9Olh19T6uYwCuUAASwl50dqdG8h+Q7EWS3uY6j2scDYwJCyRzYLSfuW3pk66jAC6wjxpju8z1R3SKZLOuM8UFRwNjwiJJ28xXJF3sOAngBAUgZnyd648ZRwOjBuyQOdN1BsAV3j1jYqe5/kWSZrvOkwhPh7JPsAuACTCSmVU51tz94xWuowCNxg6AQ7a7u0NheO4rc/0UPa/fCF2R9GTAfzaMn5VUCr8s6TTXUYBGowA02G7P6zPXn4i8lSZZ6QU2sjB+ts+c5DoD4ALvnA2wy1xfWihpkutMqcHRwKgBMzNaYO5Zutx1DqCRKAB1ZM87b46MOY+5fh1VJLsiw9HAmJgp9p7gD1fPdx0DaCTeNWvs1XN91JtdHUqbuBkQE5BV1TSXimbVco6XgjcoADVgL764oNLguyVzoazeL4m5fiNtDWRXJX8MUM5Ij85p0sOHNunx/fNaPzOrvpZQQ0XKTT3YSNqyttd1jNQyoawJgkqQCbYGWT0SBsG1TSN//JflCyhZcUEBGCfm+jFiXxwDlJP55bx+Zk6/OnWS7nxbi/pbkl9kkoIC0HhBaKJsU+b+MG8+ecMZf/tb13l8l8x3TId2mutfKGl/13mwQxKPBt48NaOlF3Tq7mNbZJMVPRUoAO4YI2WbwjW5luzZPzvt6w+7zuMrCsAoMNdPgD4j+0Aynmq1RvrlqZN1zeJOlfL8FXSFAuCeCWRzLZkf3HTONz/sOouPePfZA+b6yWNXZmJ/NPBwIdD3Lu3Sfcc0u47iPQpAfORasmtaOotzl5/Y0+86i0/i/W7ZYK/M9U23FJwv2U7XmTAGzwSyj8d3ht7XGugbn5+htQcUXEeBKABxk2kKtxTb9OafnvKtja6z+IICIMkuWnSIrF3IXD/hRozsPZlYHg1cyht97UszteagvOsoeBEFIH6yhXCbqZT2u/mC7/IH0wDJGJrWwS5zfRsx10+DXDyPBrZGuuyT07j4A2+gPFxtzxUL90g6xHUWH8TrnbLO7J//eVa9vWdJ0cWSOUUeF6DUej6QXROvMcCNZ7bp6guZJsUNOwDxVZic+f4vzv7mR13nSDsvCoA988xWFQv/XTIfkbSX6zyoo5gdDfz8tIw+83f7qJyNRx68ggIQXyaUbWoL3nzDGd9e7TpLmqX6O2Db0xPokUcukbE9kulynQcNkJE0xUob43HBvWrJFC7+wBjZqkxlOLhO0qGus6RZao8fsQsX7qfVj9wqo3/i4u8X0xW5jiBJenLfnO4/uug6BpBI5cHKm8+6/n8e7zpHmqWyANjFC06TsQ9KOsF1FjjQZqWs+0cBbj69Tdbw3T8wHtZK5Ur0Hdc50ix1BcAuXPgxRebnsmp1nQWOGEldbgtAqWC04lgO+wEmojxcPbr737tzrnOkVaoKgF103scl+11J8boNHA3negzw4BFFDRdS9dcLaLioYsPB4n4XuM6RVql5h7KLFlwgq3+QJ0824A20WKnZ3S7Aw4c1OVsbSBM7HJznOkNapaIA2MXdR8qay8XFHztxuQvwx/059AeohSiq8iRAnSS+ANgLL2xWFFwnidutsasu66wSPjeDz44CaqFa5jNZ6iXxBUDl0tfE+f14LTkrTW78GKAaGvVN4jYUoCaqlpsA6yTRBcCed95hkvkL1zkQXy7GAMMFJlFArUTWJvo6FWdJ/w/7VSX//wPqaUrU8GdCqhkKAFAz7o/0SK3EXjztokWHyugs1zkQc6GkDt5BAGB3iS0AiqKPirv+MQpmWtV1BACInUQWANvdnZPR+a5zICFicjQwAMRJIguAguAESZMdp0BSxOBoYACIm2QWAGPf6zoCksX10cAAEDfJLAAKjnWdAAnj+GhgAIibxBUA290dSvYI1zmQPGYqBQAAXpK4AqAwnCaJz1nF2E2r8twIALwoeQUgqM5wHQEJlZOTo4EBII6SVwBshrv/MW7cDAgAOySvAFSrycuM+HBwNDAAxFESL6Yc64bxCyV1sAsAAMkrAMZsch0ByWamcR8AACSvAETRetcRkHBtkZSjBADwW+IKgFm+fKtkN7rOgQQzkjgTAIDnElcAJEnW3Oc6ApKNpwEA+C6ZBcDYu1xHQMJxNDAAzyW0AIQ/dx0BycfRwAB8lsgCYJYte1DSOtc5kHAcDQzAY4ksADuYZa4TIOFykiZzLwAAPyW3AATBlZLYw8WEcCYAAF8ltgCYpUsfk7TCdQ4kXCdHAwPwU2ILgCTJ6CrXEZBwHA0MwFPJLgCV6MeSSq5jINkYAwDwUaILwIunAt7kOgcSjqOBAXgo0QVgh4AxACbGSOqiAADwS/ILQLX6C0mbXcdAsnE0MADfJL4AmOXLRyR7jescSLhmjgYG4JfEF4AdGANg4tgFAOCTVBQA8+Mfr5C02nUOJFxXxNHAALyRigKwg73adQIkXE5SG2MAAH5ITwEIMldJYg8XE8IYAIAvUlMAzNKlT0q6w3UOJBxHAwPwRGoKwA6WmwExMaF2lAAASLl0FYAwe62kQdcxkGyMAQD4IFUFwFx9da9kfuY6BxKuzUp51yEAoL5SVQAkSZYxACbISJrKLgCAdEtfAYiiX0v2OdcxkGxmGgUAQLqlrgCY5cursmap6xxIuCJHAwNIN6fnnh1/9pVdlayOstLBkjlYNjpYMl2SmiW1v/jPnMuMQNytf+IG1xGAeDOBjAIbhLmKMaZqguxgGObXB2F2tTHh/apmf7rm9q97d5psQwvA3DMuL5p84d1G5iQZnSjpsEZnANKGAgBMlFEm2zwcZptXh2HxF7ls8TurftWz1XWqeqv/xbenJ5i7av/jA+lCSQslTar7moBHKABAbQUmtJlCx+OZTPPSXGfzV1ct7xlxnake6lYATrj4h4WB/uDPZMynJc2u1zqA7ygAQP2EmWI539RxnR2oXLJ2xXd7XeeppZoXgLlnXF4MmooflbWfkjSj1q8PYFcUAKD+gjAf5Zum3BjY8INrbv/2Ztd5aqGmTwEcs+DKM4JC0ypZ+01x8QcApERULQVD/c+eOTS8ccMBJ33mW67z1EJNdgCOXfjD/arV8LuSTq/F6wEYPXYAgMbLFTo2hPkpZ6/9zd/c7TrLeE14B2DegqveX62G94uLPwDAEyPDW6eXetfdeeBJn/4711nGa9w7AAecelm+vbntGzK6tJaBAIwNOwCAW4WmrkdMKTg2aTcJjmsHYN7ZV3S2t0y+nYs/AMB3w0Mb51Qyg08fdupn3uQ6y1iMuQActXDZXjZj/p9kjq1HIAAAkqY80jupb/umVQe+53PzXWcZrTEVgLkf+OEhmWr5bkmH1ykPAACJVKkM5Et9z/z24JM/+17XWUZj1AXguO4rZoZB+EtJe9cxDwAAiVWtDGcGB5658U0nfeF411neyKgKwLyzr+gsy9xipX3rHQgAgCSLqqWwMvzsbXG/J+ANC8ABp16WtxndaKQ5jQgEAEDSVcoD+f7erb8/YP7HY/v5N29YANqb277NDX8AAIxNeaS3NSrYO1zn2JPXLQBHd19xrow+1qgwAACkSWnw+bccfOL/+IbrHK9ljwXg2IU/3M/I/J9GhgEAIG0GBp/7dBxvCtxjAahWw8skTW5gFgAAUsdGZROVt1znOsfuXrMAzFtw1fslndHgLAAApNLI8JbpB570ma+6zrGzVxWAuWdcXpS1f+8iDAAAaVUa2vDZQ0/p6XCd4yWvKgCmUPgYz/sDAFBb1cpwplTa/n9d53jJLgXggFMvyxvpE67CAACQZqXhzWce+f6eNtc5pN0KQHvz5A9KZi9XYQAASLOoWgoGtvf+o+sc0s4FoKcnkDGfdpgFAIDUK5U2L5B6xvxpvLX2coD5D89+p6TZDrMAAJB61cpQ9pB3DTk/ZO/lAhBJF7oMAgCAL0ZKffEoAMd1/3uTrDnHdRgAAHxQLm09yPUjgYEkle3Qe8SpfwAANERkq6ZcHbzUZYZAkozMSS5DAADgm2p56H0u199xD4DRiS5DAADgm0q5/1CX6wfHn31ll6TDXIYAAMA3lUp/0yEn9xzkav2gktVRkoyrAAAAeMlKVQ2939XygZUOdrU4AAA+s9HIXFdrB5KhAAAA4EAUVZxdgwNZ62z+AACAz6JKaZartQNJ010tDgCAz6wtN7taOzBSq6vFAQDwmbWVjKu1A0sBAADACRtFzj4VMJDU4mpxAAB8FtmK0wKQc7U4AABes5GzpZ01DwAA4A4FAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9RAAAA8BAFAAAAD1EAAADwEAUAAAAPUQAAAPAQBQAAAA9lXAcAACSXMaEy2WZlMi3KZHf8L8gUFIRZBUFOxoQyJpAxoeuosbXXfmfYUf7SsqR+SeslPWqNWRkac9uKOWvvUU9PNNZ1zTHdV452YQAxtP6JG1xHgDeMMtkW5QtTlG+aokxusjJhk2Rc5/LeM5J+FFSD7634yQXPjPY3UQCAhKMAoJ7CMK98cbryhSnKFToVhnnXkbBnI9bqX20281crly3e/Ea/mHsAAAC7MCZQU/Ne6pg2T117v0ttnUeoqXkvLv7xlzNGHw4qldVHn3vlwjf6xdwDAACQJGXz7Wpu2UdNzTNkgqzrOBi/TmO0bN6Cq06otg1+fOUPLim/1i9iBwAAPJcvTFXn9OM0dcbbVWzdh4t/SlhrLwm3NV0/94zLi6/17ykAAOCpfNMUTZnxdnVOP1b5whTXcVAHVjotKDT97NDuf8/t/u8YAQCAZ3KFdk3uOFzZ3GTXUdAY72qyw/8g6SM7/yQ7AADgiSDMqW3KWzRl+tu4+HvGGH34mO4rztv55ygAAJB6RsXmvdW114kqtuwjHtz3lfmnuYuWvjzroQAAQIqFmaKmzDhebVOPVBC+agwMv3QE1epXXvoBBQAAUqpQnKape71DuXyH6yiIC2s/NLf7yn0kCgAApI8JNLn9UHV0HaOAR/qwq1zw4s2AFAAASJEwU9DU6W9T8+T9xawfe3C+enoCHgMEgJTIZFvVOX2+wrDJdRTE295Hr5p9NAUAAFIgl+9Qx7R5bPljVIzMSRQAAEi4fNMUdUw9RoZNXYySkZnLVwsAJFhT80y1TT1Shlu6MAZW9mC+YgAgoQpN07j4Y7xm8FUDAAmUzberfepcLv4Yr1a+cgAgYTLZVnVOmy8ThK6jILnyFAAASJAw06TO6fO52x8TRgEAgKQwRu1T5/KcP2qCAgAACTG5/c3K5dtdx0BKUAAAIAEKTdPUPGm26xhIEQoAAMRcmCmobeqR4mx/1BIFAADizEjtU+cqCHKukyBlKAAAEGPFln2Vy3e4joEUogAAQEwFQU6T2g5xHQMpRQEAgJia1DFHQcjWP+qDAgAAMZQrtKvYsrfrGEgxCgAAxI7R5I7DxV3/qCcKAADETKHYpWxususYSDkKAADETMvkA1xHgAcoAAAQI/nCVB77Q0NQAAAgRlraDnQdAZ6gAABATGTz7coXOl3HgCcoAAAQE80t+7iOAI9QAAAgBowJ1NQ8w3UMeIQCAAAxUChOlwmyrmPAIxQAAIiBppZZriPAMxQAAHAsDPIqFLpcx4BnKAAA4Fi+ebpkOPYXjUUBAADH8oUpriPAQxQAAHDKKMez/3Ag4zoAAPgsk2tRGOZdx6ipIDA6cHaX3nrETB24f5e6OltULOYkSYODI9q4pV9r1m3U/f/1jNY+vklRZB0n9hMFAAAcyufTs/2fy4Z6zwlzdMpJh2hSS+E1f01ra0GtrQW9ab8pOvXkN6uvb1g33faIfvMfqzVSrjY4sd8oAADgUL4pHQVg3lH7avE5c9XeVhzT72ttLei8s47Su995kJZed7/ufeDJOiXE7rgHAAAcyuYmuY4wIcZI7z/1CH30T/9kzBf/nXW0Netjf/YnWnDWUQp4IqIh2AEAAEeMCRWG479ouhYYo4/86ds176h9a/J6xkjve9ehmtLRrH/+t98pstwbUE/sAACAI5lss5Tgb3Y/cMZbanbx39n8t+6nc953RM1fF7uiAACAI5lsi+sI4zbvqH11+rsPq9vrn/6ew3XMkXw6Yj1RAADAkUy22XWEccllQy065611XcMY6fwPHK1cjkl1vVAAAMCRTKbVdYRxOeXEOepoq395aW8r6l3vOLju6/iKAgAAjoSZ5B0AFARG7znxkIatd+pJcxQECb5RIsYoAADgSBDkXEcYs4P279rjIT/1MKm1oANmT23Yej6hAACAIyZMXgE46ohZDV/zrYc3fk0fUAAAwJHAhK4jjNmBDr4bZwegPigAAOCKSd5b8NQpjX90cdrUZN4sGXfJ++oDgLRIYAEoNjV+bNHUlG34mj5I3lcfAACYMAoAALhiI9cJxmxwaKThaw4NlRu+pg8oAADgSgILwKbN/Q1fc8Om3oav6QMKAAA4Etmq6whjtmbdxoavufaPmxq+pg8oAADgSBQ1fjt9ou7/r2e8WNMHFAAAcCSqJq8ArH18k/r6hhu2Xm/fsNY9vrlh6/mEAgAAjkSVkusIYxZFVjfd9kjD1rvp1ocVWduw9XxCAQAARyrlxt9QVwu33L5aW7YN1H2drdsHdesdj9Z9HV9RAADAkaQWgHKlqmU/uV/1/MbcWulH196nkXLybpRMCgoAADiS1AIgSfc+8KRuvOWhur3+z3/1oFb+4am6vT4oAADgTKUyICV4vP2TG/+gFfc/UfPXvXvlE7r+pv+q+etiVxQAAHDE2qqq1UHXMcYtslbf/7ff6qc3PViTcYC10i9+s0qXX/E7bvxrgIzrAADgs/LICwozRdcxxs1a6fqbH9Szz23X4g/MVUdb87heZ8u2AV193Uq2/RuIAgAADpWGtqhQnOE6xoTd+8BTeuChZ/Xudx6iU0+eo0mthVH9vt6+Yd1068P6zX88qnKFG/4aiQIAAA6VhtNzyE25UtVNt67SL297WAfsP1VvPXyWDpg9VdOmtqpY3PExwoODI3p+U58ee3yjfv/gs1r7+Ca2+x2hAACAQ5Vyv6JqSUGYdx2lZiJrtWbdRiefG4DR4yZAAHDKpmoXAMlBAQAAxygAcIECAACOlQaeV12P1QNeAwUAAByrRiUNDzMvR2NRAAAgBob6+cx7NBYFAABiYHhwg2xUdh0DHqEAAEAMWBtpaGC96xjwCAUAAGJioJ9jcNE4FAAAiIlyabtKw5tcx4AnKAAAECP92x9zHQGeoAAAQIyUhrdoZHib6xjwAAUAAGKm/wV2AVB/FAAAiJnhoY0aGXnBdQykHAUAAGLH6oUtD0qcDow6ogAAQAyVS9s1MMBjgagfCgAAxFTf1ocVVUdcx0BKUQAAIKaiqKze7Y+4joGUogAAQIwN9j+lkeGtrmMghSgAABBnVtq26X5GAag5CgAAxFy1OqRtW34vHgtALVEAACABSoMb1d/7R9cxkCIUAABIiL6tqzkmGDVDAQCAhLCKtG3TSlUrw66jIAUoAACQINXqkLZsuEvVqOQ6ChKOAgAACVOp9Gvr8/fIRhXXUZBcJQoAACRQubRdWzfdK9nIdRQkUx8FAAASqjS0Wds2PyBZHg/EmD1HAQCABBsaeFZbNt6tyDIOwOgZmUcpAACQcKWhzdqy4S5OC8ToGd1Hj0Hy+QAACMRJREFUAQCAFCiXtmvLht+pWhlyHQUJYI25jQIAAClRLvdr83O/00iJw4Lwup66d87alRQAAEiRanVImzf8Tv0vrBOfHYDXZHS1enoiCgAApI216t32sLY8f7eqVQ4Mwi5KGWu/J3EQEACkVmloszY/d4dGhre6joKYMLL/ctfyJc9KFAAASLVqZVibN9yp7Zse4PhgbKlmsl956QcZl0kAAI1gNTjwtIaHNmhS+xwVW/aVjOtMaDRj9ZGVyxZvfunH7AAAgCeiqKztWx7Upg2/Vbm03XUcNJLV9+659qLlO/8UOwAA4JlyaZs2PfefyhU61Np2kPKFqa4joY6s9IuWzXt/YvefpwAAgKdGhrdqy4a7lSt0qnXygco3UQRS6EY7PHTe7bef+KqzoikAAOC5keEt2jK8Rdl8m5pb9lGheS8FQdZ1LEyU1feaN+/9ide6+EsUAADAi8ql7dpe2i5tfUiF4jQVm2ep0DRNMtwxmDCbjNXHdp/5744CAADYlY00PPCchgeeUxDmVChOV6GpS7nCVAUBl40YKxnZfwlV/vJd137oDQ9/4E8SALBHUXVEg31PabDvKckY5XJtyuU7lC10KJdrVRg280ihe0/L6EcZa7/30iE/o0EBAACMjrUaKW3b8WFDvet2/JwJlMm2KJNt3vHPTIvCTF5BkFcQZmVMRjJGgeFyUwMjkvqt9Ewgs0ZG91ljbrt3ztqV6umJxvpi/IkAAMbPRqqM9Koy0us6SWI9e+9yJ3soHAQEAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4iAIAAICHKAAAAHiIAgAAgIcoAAAAeIgCAACAhygAAAB4KJA04joEAABeMu6+Dw8k9TtbHQAAjwUmE7lbW+pztTgAAD4zQeCuAFgKAAAAThiTqbhaO5C0wdXiAAD4zJjsgKu1AxmzxtXiAAD4LMjkn3G2tmQfdbU4AAA+C4KMs2twYCQKAAAADpggt9LV2kHZVu6XZF0FAADAS0bKBNFPXS0f/H75n22S9JCrAAAA+CiTaRl65JZvPuZq/RePILK3uQoAAICPMtmWVS7XDyTJKqAAAADQQGGYv9Hl+oEkZZW/RdILLoMAAOALE2RsNtvyXZcZAkm6a/mCIWN1ncsgAAD4IpfveHTVr3q2uszw8scQRTJXuQwCAIAvMtmWf3Sd4eUCcN9h6+6Q9EeHWQAASL0wUyyvubXp+65zvPJBxD09kWS+5TALAACpV8h3XiP1OPsUwJcEO/9gW/+2f5XseldhAABIsyAsVKOh8sdc55B2KwBrb760JGu+4yoMAABpVmia8rO1K77b6zqHtFsBkKSoNPR9SU80PgoAAOkVZprKzZMnf9B1jpe8qgCsvOGSQWvtR12EAQAgrQrN07/2wPU9213neMmrCoAk3Xftkpsl+/NGhwEAII2y+c71a37zjR7XOXb2mgVAkiKZj0uKTVMBACCJjMnYbLH9HNc5drfHArBy+UVPGWsvFB8VDADAOBk1t8z8xmO//voK10l2t8cCIEn3XLvkRlk5PasYAICkKhS77n/0tm991nWO1/K6BUCStg1s/0tJdzYgCwAAqZHLT36hYKa803WOPXnDArD25ktLkQqnSfpDA/IAAJB4Ybal1NzaPnfV7T39rrPsyRsWAElauXzBCxnZ9xnpyXoHAgAgyYKwUM0V9zrhoZu/uc51ltczqgIgSXctX/JsJHOKpKfqmAcAgMQKM4VKc+vM09b+5m/udp3ljYy6AEjSfcsvfDSqmmOt9GC9AgEAkESZbHMp3zrr7atv+d+/dp1lNMZUACRp5U8ufC6rkRPFjYEAAEjaccPf5M5Zc+L4uN+ejLkASNJdyz+0tXnT3u+UtX8rzgkAAHjLKF+c9odiZtasB2/468ddpxkLM9EXOPrcK84yxvxQUnsN8gAYo/VP3OA6AuClwIS2qWXm36+57dufcp1lPMa1A7Cz+65d8rNKNTyKzw4AAPgim+9cX2ibfVxSL/5SDXYAdjbv3CtOt8ZcJml2LV8XwJ6xAwA0TpgplgvN074Wtw/2GY8J7wDs7J5rl9yYUeFQa/VJSc/W8rUBAHAlCAvVYvOsn3ZMm92Vhou/VOMdgJ0dcOpl+bbmyRcbYz4j6U31WgfwHTsAQP2EmWK5kO+8JqtJH4nzqX7jUbcCsLP53f82N7LBRTI6X1JnI9YEfEEBAGrLBBmbzbc/nsm2Xv7YrU3fknoi15nqoSEF4CUnXPzDwuBA5mRZe7I15kTJHqEajyEA31AAgAkyUibTMpTJtjwchvkbipNa/uGB63u2u45Vbw0tALubu2jpFFOpHiVjDwqsPcTKHCypS1KrpDZJLZJyLjMCcUcBAN6ACWRMaIMgWzHGVIzJDYaZ7LMmyK02QW5lJoh++sgt33zMdcxG+/8aRz9NejfySgAAAABJRU5ErkJggg=='
  }

  getDescription(): string {
    return 'Local Wallet'
  }

  getInstallUrl(): string {
    return ''
  }

  async signTransaction(input: Transaction): Promise<Authenticator> {
    return this.keypair!.signTransaction(input)
  }

  async sign(msg: Bytes): Promise<Bytes> {
    return this.keypair!.sign(msg)
  }

  getTarget(): any {
    return this
  }

  async connect(): Promise<ThirdPartyAddress[]> {
    const accounts = LocalWallet.getAccounts()
    const activeAddress = window.localStorage.getItem(LocalActiveAddress)
    const network = await this.getNetwork()
    for (const account of accounts.entries()) {
      for (const key of account[1].keys) {
        if (activeAddress === key) {
          this.keypair = Secp256k1Keypair.fromSecretKey(key)
          break
        }
      }
    }

    this.currentAddress = this.keypair!.getBitcoinAddressWith(this.formatNetwork(network))
    this.publicKey = this.keypair!.getSchnorrPublicKey().toString()
    this.address = [this.currentAddress]
    return this.address
  }

  switchNetwork(network: WalletNetworkType): Promise<void> {
    window.localStorage.setItem('local-wallet-net', network)
    if (this.onNetworkChange) {
      this.onNetworkChange(network)
    }
    if (this.onAccountsChange) {
      this.onAccountsChange([
        this.keypair!.getBitcoinAddressWith(this.formatNetwork(network)).toStr(),
      ])
    }
    return Promise.resolve()
  }

  getNetwork(): Promise<WalletNetworkType> {
    return Promise.resolve(
      (window.localStorage.getItem('local-wallet-net') as WalletNetworkType) || 'testnet',
    )
  }

  getSupportNetworks(): WalletNetworkType[] {
    return ['livenet', 'testnet']
  }

  onAccountsChanged(callback: (account: string[]) => void): void {
    this.onAccountsChange = callback
  }

  removeAccountsChanged(_: (account: string[]) => void): void {
    this.onAccountsChange = undefined
  }

  // TODO: support listener?
  onNetworkChanged(callback: (network: string) => void): void {
    this.onNetworkChange = callback
  }

  removeNetworkChanged(_: (network: string) => void): void {
    this.onNetworkChange = undefined
  }

  sendBtc(_: {
    toAddress: string
    satoshis: number
    options?: { feeRate: number }
  }): Promise<string> {
    throw Error('local not support sendBtc!')
  }

  getBalance(): Promise<{ confirmed: number; unconfirmed: number; total: string }> {
    throw Error('local not support getBalance!')
  }

  private formatNetwork(network: WalletNetworkType) {
    switch (network) {
      case 'livenet':
        return BitcoinNetowkType.Bitcoin
      default:
        return BitcoinNetowkType.Testnet
    }
  }
}
