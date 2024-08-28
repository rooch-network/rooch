export enum TransactionStatus {
  Executed = 'executed',
  OutOfGas = 'outofgas',
  MoveAbort = 'moveabort',
  ExecutionFailure = 'executionfailure',
  MiscellaneousError = 'miscellaneouserror',
}

export enum TransactionType {
  L1_BLOCK = 'l1_block',
  L1_TX = 'l1_tx',
  L2_TX = 'l2_tx',
}
