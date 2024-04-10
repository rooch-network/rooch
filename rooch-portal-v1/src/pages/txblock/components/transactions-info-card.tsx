import React from 'react'
import { TransactionCheckpoint } from '@/common/interface'

type TransactionInfoCardProps = {
  transaction: TransactionCheckpoint
}

const TransactionInfoCard: React.FC<TransactionInfoCardProps> = ({ transaction }) => {
  return (
    <div className="bg-inherit overflow-hidden">
      <div className=" text-left">
        <div className="uppercase tracking-wide text-sm text-indigo-500 font-semibold">
          {transaction.action}
        </div>
        <p className="block mt-1 text-lg leading-tight font-medium text-muted-foreground">
          Amount:{' '}
          <span className="font-bold">
            {transaction.amount} {transaction.currency}
          </span>
        </p>
        <p className="mt-2 text-muted-foreground">
          Sender: <span className="text-gray-500">{transaction.sender}</span>
        </p>
        <p className="text-muted-foreground">
          Recipients: <span className="text-gray-500">{transaction.recipients}</span>
        </p>
        <p className="text-muted-foreground">
          Status:{' '}
          <span
            className={`font-bold ${
              transaction.status === 'Success' ? 'text-green-500' : 'text-red-500'
            }`}
          >
            {transaction.status}
          </span>
        </p>
        <div className="mt-4">
          <h4 className="text-gray-500 font-semibold">Fees:</h4>
          <p className="text-muted-foreground">
            Total Gas Fee: <span className="text-gray-500">{transaction.totalGasFee}</span>
          </p>
          <p className="text-muted-foreground">
            Computation Fee: <span className="text-gray-500">{transaction.computationFee}</span>
          </p>
          <p className="text-muted-foreground">
            Storage Fee: <span className="text-gray-500">{transaction.storageFee}</span>
          </p>
          <p className="text-muted-foreground">
            Storage Rebate: <span className="text-gray-500">{transaction.storageRebate}</span>
          </p>
          <p className="text-muted-foreground">
            Gas Payment: <span className="text-gray-500">{transaction.gasPayment}</span>
          </p>
          <p className="text-muted-foreground">
            Gas Budget: <span className="text-gray-500">{transaction.gasBudget}</span>
          </p>
          <p className="text-muted-foreground">
            Gas Price: <span className="text-gray-500">{transaction.gasPrice}</span>
          </p>
        </div>
      </div>
    </div>
  )
}

export default TransactionInfoCard
