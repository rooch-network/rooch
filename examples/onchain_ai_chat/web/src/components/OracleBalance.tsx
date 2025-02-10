import { useRoochClient, useRoochClientQuery, useCurrentSession } from '@roochnetwork/rooch-sdk-kit';
import { useNetworkVariable } from '../networks';
import { WalletIcon, ArrowDownCircleIcon } from '@heroicons/react/24/outline';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useState } from 'react';

// Create a custom hook for balance query
export function useOracleBalance() {
  const sessionKey = useCurrentSession();
  const packageId = useNetworkVariable('packageId');

  return useRoochClientQuery(
    'executeViewFunction',
    {
      target: `${packageId}::ai_service::get_user_oracle_fee_balance`,
      args: [Args.address(sessionKey?.roochAddress.toHexAddress() || '0x0')],
    },
    {
      enabled: !!sessionKey,
      refetchOnMount: true,
      refetchInterval: 10000,
    }
  );
}

export function OracleBalance() {
  const sessionKey = useCurrentSession();
  const client = useRoochClient();
  const packageId = useNetworkVariable('packageId');
  const { data: balanceResponse, refetch: refetchBalance } = useOracleBalance();
  const [withdrawing, setWithdrawing] = useState(false);

  if (!sessionKey) return null;

  const balance = balanceResponse?.return_values?.[0]?.decoded_value
    ? Number(balanceResponse.return_values[0].decoded_value) / 100_000_000
    : 0;

  const handleWithdraw = async () => {
    if (!client || !sessionKey || balance <= 0) return;
    
    try {
      setWithdrawing(true);
      const tx = new Transaction();
      tx.callFunction({
        target: `${packageId}::ai_service::withdraw_all_user_oracle_fee`,
        args: [],
      });
      
      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey,
      });

      // Check transaction execution status
      if (result.execution_info.status.type !== 'executed') {
        throw new Error('Failed to withdraw: transaction failed');
      }
      
      await refetchBalance();
    } catch (error) {
      console.error('Failed to withdraw balance:', error);
      // You might want to show an error notification here
    } finally {
      setWithdrawing(false);
    }
  };

  return (
    <div className="fixed bottom-4 left-4">
      <div className="bg-white shadow-lg rounded-lg p-4">
        <h2 className="text-sm font-semibold text-gray-500 mb-2 flex items-center gap-2">
          <WalletIcon className="h-5 w-5 text-gray-400" />
          ORACLE BALANCE
        </h2>
        <div className="flex items-center justify-between gap-4">
          <div className="text-sm font-medium text-gray-900">
            {balance.toFixed(4)} <span className="text-gray-500">RGas</span>
          </div>
          {balance > 0 && (
            <button
              onClick={handleWithdraw}
              disabled={withdrawing}
              className={`flex items-center gap-1 text-xs px-2 py-1 rounded-md 
                ${withdrawing 
                  ? 'bg-gray-100 text-gray-400' 
                  : 'bg-blue-50 text-blue-600 hover:bg-blue-100'}`}
            >
              <ArrowDownCircleIcon className="h-4 w-4" />
              {withdrawing ? 'Withdrawing...' : 'Withdraw'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}