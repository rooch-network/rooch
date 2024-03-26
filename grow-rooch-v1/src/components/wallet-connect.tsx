import { useWalletStore } from '@roochnetwork/rooch-sdk-kit'

export const WalletConnect = () => {
  const account = useWalletStore((state) => state.currentAccount)

  if (!account) {
    return <div>No Account</div>
  } else {
    return <div>{account?.getAddress()}</div>
  }
}
