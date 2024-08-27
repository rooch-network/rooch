// import { Args, Transaction, normalizeTypeArgsToStr } from '@roochnetwork/rooch-sdk';
// import {
//   useCurrentAddress,
//   useRoochClientQuery,
//   UseSignAndExecuteTransaction,
// } from '@roochnetwork/rooch-sdk-kit';

// import {
//   Stack,
//   Button,
//   Dialog,
//   Select,
//   MenuItem,
//   TextField,
//   InputLabel,
//   DialogTitle,
//   FormControl,
//   DialogActions,
//   DialogContent,
//   FormHelperText,
// } from '@mui/material';

// import { useRccList } from 'src/hooks/data/useRccList';

// import { RED_ENVELOPE } from 'src/config/constant';

// export default function CoinTransferModal({
//   open,
//   onClose,
// }: {
//   open: boolean;
//   onClose: () => void;
// }) {
//   const { mutateAsync: signAndExecuteTransaction } = UseSignAndExecuteTransaction();
//   const currentAddress = useCurrentAddress();

//   const { data: RCCBalance } = useRoochClientQuery('getBalance', {
//     owner: currentAddress?.genRoochAddress().toStr() || '',
//     coinType:
//       '0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812::rooch_clicker_coin::RCC',
//   });

//   console.log(
//     '🚀 ~ file: send-coin-envelope-modal.tsx:42 ~ SendCoinEnvelopeModal ~ RCCBalance:',
//     RCCBalance
//   );

//   const { rccOwnerList } = useRccList(currentAddress?.genRoochAddress().toStr());
//   console.log(
//     '🚀 ~ file: send-coin-envelope-modal.tsx:47 ~ SendCoinEnvelopeModal ~ rccOwnerList:',
//     rccOwnerList
//   );

//   return (
//     <Dialog open={open}>
//       <DialogTitle sx={{ pb: 2 }}>Send Coin Envelope</DialogTitle>

//       <DialogContent
//         sx={{
//           width: '480px',
//           overflow: 'unset',
//         }}
//       >
//         <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1 }}>
//           <FormControl>
//             <InputLabel>Items per</InputLabel>
//             <Select
//               value={10}
//               // variant="filled"
//               color="secondary"
//               label="Item per 1"
//               // onChange={handleChange}
//             >
//               <MenuItem value={10}>Ten</MenuItem>
//               <MenuItem value={20}>Twenty</MenuItem>
//               <MenuItem value={30}>Thirty</MenuItem>
//             </Select>
//           </FormControl>
//           <FormControl>
//             <InputLabel>Items per</InputLabel>
//             <TextField />
//           </FormControl>
//         </Stack>

//         {false && (
//           <FormHelperText error sx={{ px: 2 }}>
//             End date must be later than start date
//           </FormHelperText>
//         )}
//       </DialogContent>

//       <DialogActions>
//         <Button
//           variant="outlined"
//           color="inherit"
//           onClick={() => {
//             onClose();
//           }}
//         >
//           Cancel
//         </Button>

//         <Button
//           disabled={false}
//           variant="contained"
//           onClick={async () => {
//             if (!rccOwnerList) {
//               return;
//             }
//             const txn = new Transaction();
//             txn.callFunction({
//               address: RED_ENVELOPE,
//               module: 'red_envelope_v3',
//               function: 'create_coin_envelope',
//               args: [
//                 // claim_type: 0-EQUAL;1-RANDOM
//                 Args.u8(0),
//                 // total_envelope_num
//                 Args.u64(20n),
//                 // total_coin
//                 Args.u256(200n),
//                 // start_time
//                 Args.u64(1722157816000n),
//                 // end_time
//                 Args.u64(1733157816000n),
//                 // coin
//                 // Args.objectId('0x9b4d3e2e37c0d39fcff8c0056a54e86c97660147d40fa2ea6a6136884196d92d'),
//               ],
//               typeArgs: [
//                 normalizeTypeArgsToStr({
//                   target:
//                     '0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812::rooch_clicker_coin::RCC',
//                 }),

//                 //   {
//                 //   // target:
//                 //   //   '0x3::coin_store::CoinStore<0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812::rooch_clicker_coin::RCC>',
//                 //   // address: '0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812',
//                 //   // module: 'rooch_clicker_coin',
//                 //   // name: 'RCC',
//                 //   // target:
//                 //   //   '0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812::rooch_clicker_coin::RCC',
//                 // }
//               ],
//             });
//             await signAndExecuteTransaction({ transaction: txn });
//           }}
//         >
//           Apply
//         </Button>
//       </DialogActions>
//     </Dialog>
//   );
// }
