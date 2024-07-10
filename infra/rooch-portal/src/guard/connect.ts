// // Copyright (c) RoochNetwork
// // SPDX-License-Identifier: Apache-2.0
// import { ReactNode, useEffect, useState } from 'react'
// import { useLocation } from 'react-router-dom'
// import {
//   useCreateSessionKey,
//   useCurrentSession,
//   useCurrentWallet,
// } from '@roochnetwork/rooch-sdk-kit'
// import { navItems } from '@/navigation'
//
// interface SessionGuardProps {
//   children: ReactNode
// }
//
// export const ConnectGuard = (props: SessionGuardProps) => {
//   const { children } = props
//
//   const { isConnected } = useCurrentWallet()
//   const [open, setOpen] = useState(false)
//   const [error, setError] = useState<string | null>(null)
//
//   const sessionKey = useCurrentSession()
//   const { mutateAsync: createSessionKey } = useCreateSessionKey()
//
//   const s = useLocation()
//
//   useEffect(() => {
//     if (!isConnected) {
//       return
//     }
//
//     const a = sessionKey === null &&
//       navItems().find((item) => s.pathname.startsWith(item.path) && item.auth) !== undefined
//     console.log(a)
//
//     setOpen(
//       sessionKey === null &&
//       navItems().find((item) => s.pathname.startsWith(item.path) && item.auth) !== undefined,
//     )
//   }, [isConnected, s, sessionKey])
//
//   const handleAuth = async () => {
//     setError(null)
//     try {
//       await createSessionKey({
//         appName: 'rooch-portal',
//         appUrl: 'portal.rooch.network',
//         scopes: defaultScope,
//       })
//     } catch (e) {
//       console.log(e)
//       setError(
//         'Authorization failed due to insufficient gas fees. Please ensure you have enough gas fees.',
//       )
//     }
//   }
//
//   return (
//     <>
//       <SessionKeyModal isOpen={open} onAuthorize={handleAuth} scopes={defaultScope} error={error} />
//   {children}
//   </>
// )
// }
