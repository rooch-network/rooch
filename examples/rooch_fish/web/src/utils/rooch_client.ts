import { TransactionWithInfoView, RoochClient, Transaction, Bytes, Signer, ExecuteTransactionResponseView, str } from "@roochnetwork/rooch-sdk";
 
export const listFieldStates = async (client: RoochClient, object_id: string, stateRoot?: string | null) => {
  try {
    let result: any[] = [];
    let cursor = null;
    // eslint-disable-next-line no-constant-condition
    while (true) {
      try {
        const data = await (client as any).transport.request({
          method: 'rooch_listFieldStates',
          params: [
            object_id, 
            cursor, 
            "100", 
            {
              decode: true,
              stateRoot: stateRoot,
            },
          ],
        }) as any;

        //console.log("🚀 ~ file: listFieldStates ~ data:", data);
        
        if (!data) {
          throw new Error('No data returned from listFieldStates request');
        }

        cursor = data.next_cursor ?? null;
        result = result.concat(data.data || []);
        
        if (!data.has_next_page) {
          break;
        }
      } catch (error) {
        console.error('Error during listFieldStates pagination:', error);
        break; // Break the loop on error
      }
    }

    return { result };
  } catch (error) {
    console.error('Fatal error in listFieldStates:', error);
    throw error; // Re-throw to be handled by React Query's error handling
  }
};

export const syncStates = async (client: RoochClient, object_id: string, txOrder?: string | null) => {
  try {
    let result: any[] = [];
    let cursor = null;
    // eslint-disable-next-line no-constant-condition
    while (true) {
      try {
        const data = await (client as any).transport.request({
          method: 'rooch_syncStates',
          params: [
            {
              object_i_d: object_id,
            }, 
            txOrder, 
            "5", 
            {
              decode: true,
              descending: false,
            },
          ],
        }) as any;

        //console.log("🚀 ~ file: listFieldStates ~ data:", data);
        
        if (!data) {
          throw new Error('No data returned from listFieldStates request');
        }

        cursor = data.next_cursor ?? null;
        result = result.concat(data.data || []);
        
        if (!data.has_next_page) {
          break;
        }
      } catch (error) {
        console.error('Error during listFieldStates pagination:', error);
        break; // Break the loop on error
      }
    }

    return { result, cursor };
  } catch (error) {
    console.error('Fatal error in listFieldStates:', error);
    throw error; // Re-throw to be handled by React Query's error handling
  }
}

export const getTransactionsByOrder = async (client: RoochClient, cursor: number | null, limit: number | null, descending_order: boolean) => {
  try {
    let result: TransactionWithInfoView[] = [];
    // eslint-disable-next-line no-constant-condition
    while (true) {
      try {
        const data = await (client as any).transport.request({
          method: 'rooch_getTransactionsByOrder',
          params: [
            cursor, 
            limit, 
            descending_order,
          ],
        }) as any;

        //console.log("🚀 ~ file: listFieldStates ~ data:", data);
        
        if (!data) {
          throw new Error('No data returned from listFieldStates request');
        }

        cursor = data.next_cursor ?? null;
        result = result.concat(data.data || []);
        
        if (!data.has_next_page) {
          break;
        }
      } catch (error) {
        console.error('Error during listFieldStates pagination:', error);
        break; // Break the loop on error
      }
    }

    return { result, cursor };
  } catch (error) {
    console.error('Fatal error in listFieldStates:', error);
    throw error; // Re-throw to be handled by React Query's error handling
  }
};

export const getLatestTransaction = async (client: RoochClient) => {
  try {
    let cursor = null;

    while (true) {
      const resp = await getTransactionsByOrder(client, cursor, 100, true)
      //console.log("getLatestTransaction resp:", resp)
      const txs = Array.from(resp.result || []).filter((item) => item.execution_info !== null);
      cursor = resp.cursor;

      if (txs.length > 0) {
       return txs[0]
      }

      // Sleep for 1 second using Promise
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  } catch (error) {
    console.error('Fatal error in getLatestTransaction:', error);
    throw error; // Re-throw to be handled by React Query's error handling
  }
};

export const signAndExecuteTransactionX = async({
  client,
  transaction,
  seqNumber,
  signer,
  option = { withOutput: true },
}: {
  client: RoochClient
  transaction: Transaction | Bytes
  seqNumber: number
  signer: Signer
  option?: {
    withOutput: boolean
  }
}): Promise<ExecuteTransactionResponseView> =>  {
  let transactionHex: string

  if (transaction instanceof Uint8Array) {
    transactionHex = str('hex', transaction)
  } else {
    let sender = signer.getRoochAddress().toHexAddress()
    transaction.setChainId(await client.getChainId())
    transaction.setSeqNumber(BigInt(seqNumber))
    transaction.setSender(sender)

    const auth = await signer.signTransaction(transaction)

    transaction.setAuth(auth)

    transactionHex = `0x${transaction.encode().toHex()}`
  }

  return await (client as any).transport.request({
    method: 'rooch_executeRawTransaction',
    params: [transactionHex, option],
  })
}
