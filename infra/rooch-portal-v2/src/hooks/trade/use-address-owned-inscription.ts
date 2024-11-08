export interface InscriptionObject {
  data: {
    objectId: string;
    version: string;
    digest: string;
    content: {
      dataType: string;
      type: string;
      hasPublicTransfer: boolean;
      fields: {
        acc: string;
        amount: string;
        attach_coin: string;
        id: {
          id: string;
        };
        metadata?: {
          type: string;
          fields: {
            content: number[];
            content_type: string;
          };
        };
        tick: string;
      };
    };
  };
}

export default function useAddressOwnedInscription(address?: string) {
  // const {
  //   data: ownedObjects,
  //   isRefetching,
  //   isFetching,
  //   refetch,
  // } = useQuery({
  //   queryKey: [`${address}-OwnedInscription`],
  //   queryFn: async () => await getAddressFullOwnedObjects(address),
  //   enabled: Boolean(address),
  //   staleTime: 30 * 1000,
  // });

  // const tickCollection = useMemo(() => {
  //   const res: { [key: string]: InscriptionObject[] } = {};
  //   if (!ownedObjects) {
  //     return res;
  //   }

  //   for (let index = 0; index < ownedObjects.length; index++) {
  //     const item = ownedObjects[index];
  //     if (!res[item.data.content.fields.tick]) {
  //       res[item.data.content.fields.tick] = [];
  //       res[item.data.content.fields.tick].push(item);
  //     } else {
  //       res[item.data.content.fields.tick].push(item);
  //     }
  //   }

  //   return res;
  // }, [ownedObjects, ownedObjects?.length]);

  return {
    userInscription: [],
    tickCollection: {},
    isFetching: false,
    refetch: () => {},
  };
}
