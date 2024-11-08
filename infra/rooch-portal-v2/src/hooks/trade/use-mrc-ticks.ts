// import type { Tick_V2 } from 'src/sections/mrc-list/view';

// import { getSuiDynamicFields } from 'src/sections/mrc-list/utils';

// export default function useMRCTicks() {
//   const { data: ticks, isFetching } = useQuery({
//     queryKey: ['query-ticks'],
//     queryFn: async () => {
//       const res: Tick_V2[] = await getSuiDynamicFields(
//         NETWORK_PACKAGE[NETWORK].DEPLOY_RECORD,
//         'record'
//       );
//       return res.sort((a, b) => Number(a.start_time_ms) - Number(b.start_time_ms));
//     },
//     staleTime: 30 * 1000,
//   });
//   console.log('🚀 ~ file: useMRCTicks.ts:17 ~ useMRCTicks ~ ticks:', ticks);
//   const tickRecordMap = useMemo(() => {
//     const res: {
//       [key: string]: string;
//     } = {};
//     ticks?.forEach((tick) => {
//       res[tick.tick.toLowerCase()] = tick.id.id;
//     });
//     res.upup = '0xa5b8d247d8df6062cff537e62610839412818e5492933327bb08d50bee61cb97';
//     return res;
//   }, [ticks, ticks?.length]);
//   console.log('🚀 ~ file: useMRCTicks.ts:28 ~ useMRCTicks ~ ticks:', ticks);

//   return {
//     tickList: ticks,
//     isFetching,
//     tickRecordMap,
//   };
// }
