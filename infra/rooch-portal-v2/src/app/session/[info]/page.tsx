"use client";

import { Box, Card, CardContent, CardHeader, Chip, Stack } from '@mui/material'
import { LoadingButton } from '@mui/lab'
import { Ed25519Keypair, fromB64, Session } from '@roochnetwork/rooch-sdk';
import { DashboardContent } from '../../../layouts/dashboard'
import { useCreateSessionKey } from '@roochnetwork/rooch-sdk-kit';

export default function Page({ params }: { params: { info: string } }) {
  console.log(params.info)
  const decoder = new TextDecoder();
  const jsonInfo = decoder.decode(fromB64(params.info));
  const { mutateAsync: createSessionKey } = useCreateSessionKey();

  console.log(jsonInfo)

  return (<></>)
  // const {
  //   appName,
  //   appUrl,
  //   scopes,
  //   secretKey,
  //   maxInactiveInterval,
  //   bitcoinAddress,
  //   roochAddress,
  //   localCreateSessionTime,
  //   lastActiveTime,
  // } = JSON.parse(jsonInfo)
  //
  // const handleCreateSession = async () => {
  //
  //   const s = await createSessionKey({
  //     appName,
  //     appUrl,
  //     scopes,
  //     maxInactiveInterval,
  //     keypair: Ed25519Keypair.fromSecretKey(secretKey)
  //   })
  //
  //   console.log(s?.getAuthKey())
  //   console.log('-----------create session is ok?', s)
  // }
  // return (
  //   <DashboardContent maxWidth="xl">
  //     <Card>
  //       <CardHeader title="Hello TG" sx={{ mb: 1 }} />
  //       <CardContent className="!pt-0">
  //         <Stack spacing={2}>
  //           <Stack direction="row" alignItems="center" spacing={0.5}>
  //             <Chip className="w-fit" label="App Name:" variant="soft" color="default" />
  //             <Box className="text-gray-400 text-sm font-medium">{appName}</Box>
  //           </Stack>
  //           <Stack direction="row" alignItems="center" spacing={0.5}>
  //             <Chip className="w-fit" label="App Url:" variant="soft" color="default" />
  //             <Box className="text-gray-400 text-sm font-medium">{appUrl}</Box>
  //           </Stack>
  //           <Stack direction="row" alignItems="center" spacing={0.5}>
  //             <Chip className="w-fit" label="Scopes:" variant="soft" color="default" />
  //             <Box className="text-gray-400 text-sm font-medium">{scopes}</Box>
  //           </Stack>
  //           <Stack direction="row" alignItems="center" spacing={0.5}>
  //             <Chip className="w-fit" label="Address:" variant="soft" color="default" />
  //             <Box className="text-gray-400 text-sm font-medium">{bitcoinAddress}</Box>
  //           </Stack>
  //           <Stack direction="row" alignItems="center" spacing={0.5}>
  //             <Chip className="w-fit" label="Max Inactive Interval:" variant="soft" color="default" />
  //             <Box className="text-gray-400 text-sm font-medium">{maxInactiveInterval}</Box>
  //           </Stack>
  //
  //           <LoadingButton
  //             variant="soft"
  //             color="primary"
  //             onClick={() => {
  //               handleCreateSession()
  //               console.log(params)
  //             }}
  //           >
  //             Create
  //           </LoadingButton>
  //         </Stack>
  //       </CardContent>
  //     </Card>
  //   </DashboardContent>
  // );
}

// http://localhost:8083/session/eyJhcHBOYW1lIjoicm9vY2hfdGVzdCIsImFwcFVybCI6Imh0dHBzOi8vdGVzdC5jb20iLCJzY29wZXMiOlsiMHhmOWIxMGU2Yzc2MGYxY2FkY2U5NWM2NjRiM2EzZWFkM2M5ODViYmU5ZDYzYmQ1MWE5YmYxNzYwNzg1ZDI2YTFiOjoqOjoqIiwiMHgzOjpzZXNzaW9uX2tleTo6cmVtb3ZlX3Nlc3Npb25fa2V5X2VudHJ5Il0sInNlY3JldEtleSI6InJvb2Noc2VjcmV0a2V5MXF6cnp2aHJranl6d3NxNHMycDYzbTJjenZ1czJ4OTZnNXZ2M3MyajI5OHE2NjJmbnFqbTNnenA0Z2tqIiwibWF4SW5hY3RpdmVJbnRlcnZhbCI6MTIwMCwiYml0Y29pbkFkZHJlc3MiOiJ0YjFxeHZyemRxbG5tcHp4cjZ6c2c3ZzJjNjJndTZsMzNxeHp6Nno1bDIiLCJyb29jaEFkZHJlc3MiOiJyb29jaDFnOXVqZDJtYzUzbmFtcWdneGM1eDhoaDNtY3JrYXdtZnRxanVhZDg4eGc0enJ4NHNhcDdzdm5wdHBsIiwibG9jYWxDcmVhdGVTZXNzaW9uVGltZSI6MTczMDA1MjM3NDU1NCwibGFzdEFjdGl2ZVRpbWUiOjE3MzAwNTIzNzQ1NTR9
// {"appName":"rooch_test","appUrl":"https://test.com","scopes":["0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a1b::*::*","0x3::session_key::remove_session_key_entry"],"secretKey":"roochsecretkey1qzrzvhrkjyzwsq4s2p63m2czvus2x96g5vv3s2j298q662fnqjm3gzp4gkj","maxInactiveInterval":1200,"bitcoinAddress":"tb1qxvrzdqlnmpzxr6zsg7g2c62gu6l33qxzz6z5l2","roochAddress":"rooch1g9ujd2mc53namqggxc5x8hh3mcrkawmftqjuad88xg4zrx4sap7svnptpl","localCreateSessionTime":1730052374554,"lastActiveTime":1730052374554}