import { useContext } from 'react'
import { MetamaskContext } from 'src/context/wallet/MetamaskContext'

export const useMetamask = () => useContext(MetamaskContext)
