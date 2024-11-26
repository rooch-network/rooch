package types

// BroadcastTXParams represents parameters for broadcasting a Bitcoin transaction
type BroadcastTXParams struct {
    Hex           string  `json:"hex"`
    MaxFeeRate    *int    `json:"maxfeerate,omitempty"`
    MaxBurnAmount *int    `json:"maxburnamount,omitempty"`
}

// QueryInscriptionsParams represents parameters for querying inscriptions
type QueryInscriptionsParams struct {
    Filter          InscriptionFilterView  `json:"filter"`
    Cursor          *IndexerStateIDView    `json:"cursor,omitempty"`
    Limit           *string                `json:"limit,omitempty"`
    DescendingOrder *bool                  `json:"descendingOrder,omitempty"`
}

// QueryUTXOsParams represents parameters for querying UTXOs
type QueryUTXOsParams struct {
    Filter          UTXOFilterView      `json:"filter"`
    Cursor          *IndexerStateIDView `json:"cursor,omitempty"`
    Limit           *string             `json:"limit,omitempty"`
    DescendingOrder *bool               `json:"descendingOrder,omitempty"`
}

// DryRunRawTransactionParams represents parameters for dry running a raw transaction
type DryRunRawTransactionParams struct {
    TxBcsHex string `json:"txBcsHex"`
}

// ExecuteRawTransactionParams represents parameters for executing a raw transaction
type ExecuteRawTransactionParams struct {
    TxBcsHex string     `json:"txBcsHex"`
    TxOption *TxOptions `json:"txOption,omitempty"`
}

// ExecuteViewFunctionParams represents parameters for executing a view function
type ExecuteViewFunctionParams struct {
    FunctionCall FunctionCallView `json:"functionCall"`
}

// GetBalanceParams represents parameters for getting account balance
type GetBalanceParams struct {
    Owner    string `json:"owner"`
    CoinType string `json:"coinType"`
}

// GetBalancesParams represents parameters for getting account balances
type GetBalancesParams struct {
    Owner    string             `json:"owner"`
    Cursor   *IndexerStateIDView `json:"cursor,omitempty"`
    Limit    *string            `json:"limit,omitempty"`
}

// GetChainIDParams represents empty parameters for getting chain ID
type GetChainIDParams struct {}

// GetEventsByEventHandleParams represents parameters for getting events by handle
type GetEventsByEventHandleParams struct {
    EventHandleType  string        `json:"eventHandleType"`
    Cursor          *string        `json:"cursor,omitempty"`
    Limit           *string        `json:"limit,omitempty"`
    DescendingOrder *bool          `json:"descendingOrder,omitempty"`
    EventOptions    *EventOptions  `json:"eventOptions,omitempty"`
}

// GetFieldStatesParams represents parameters for getting object field states
type GetFieldStatesParams struct {
    ObjectID     string         `json:"objectId"`
    FieldKey     []string       `json:"fieldKey"`
    StateOption  *StateOptions  `json:"stateOption,omitempty"`
}

// GetModuleABIParams represents parameters for getting module ABI
type GetModuleABIParams struct {
    ModuleAddr string `json:"moduleAddr"`
    ModuleName string `json:"moduleName"`
}

// GetObjectStatesParams represents parameters for getting object states
type GetObjectStatesParams struct {
    ObjectIDs    string        `json:"objectIds"`
    StateOption  *StateOptions `json:"stateOption,omitempty"`
}

// GetStatesParams represents parameters for getting states
type GetStatesParams struct {
    AccessPath  string        `json:"accessPath"`
    StateOption *StateOptions `json:"stateOption,omitempty"`
}

// GetTransactionsByHashParams represents parameters for getting transactions by hash
type GetTransactionsByHashParams struct {
    TxHashes []string `json:"txHashes"`
}

// GetTransactionsByOrderParams represents parameters for getting transactions by order
type GetTransactionsByOrderParams struct {
    Cursor          *string `json:"cursor,omitempty"`
    Limit           *string `json:"limit,omitempty"`
    DescendingOrder *bool   `json:"descendingOrder,omitempty"`
}

// ListFieldStatesParams represents parameters for listing field states
type ListFieldStatesParams struct {
    ObjectID     string        `json:"objectId"`
    Cursor       *string       `json:"cursor,omitempty"`
    Limit        *string       `json:"limit,omitempty"`
    StateOption  *StateOptions `json:"stateOption,omitempty"`
}

// ListStatesParams represents parameters for listing states
type ListStatesParams struct {
    AccessPath   string        `json:"accessPath"`
    Cursor       *string       `json:"cursor,omitempty"`
    Limit        *string       `json:"limit,omitempty"`
    StateOption  *StateOptions `json:"stateOption,omitempty"`
}

// QueryEventsParams represents parameters for querying events
type QueryEventsParams struct {
    Filter      EventFilterView        `json:"filter"`
    Cursor      *IndexerEventIDView    `json:"cursor,omitempty"`
    Limit       *string                `json:"limit,omitempty"`
    QueryOption *QueryOptions          `json:"queryOption,omitempty"`
}

// QueryObjectStatesParams represents parameters for querying object states
type QueryObjectStatesParams struct {
    Filter      ObjectStateFilterView  `json:"filter"`
    Cursor      *IndexerStateIDView    `json:"cursor,omitempty"`
    Limit       *string                `json:"limit,omitempty"`
    QueryOption *QueryOptions          `json:"queryOption,omitempty"`
}

// QueryTransactionsParams represents parameters for querying transactions
type QueryTransactionsParams struct {
    Filter      TransactionFilterView  `json:"filter"`
    Cursor      *string                `json:"cursor,omitempty"`
    Limit       *string                `json:"limit,omitempty"`
    QueryOption *QueryOptions          `json:"queryOption,omitempty"`
}

// RepairIndexerParams represents parameters for repairing indexer
type RepairIndexerParams struct {
    RepairType   string                   `json:"repairType"`
    RepairParams RepairIndexerParamsView  `json:"repairParams"`
}

// SendRawTransactionParams represents parameters for sending raw transaction
type SendRawTransactionParams struct {
    TxBcsHex string `json:"txBcsHex"`
}

// StatusParams represents empty parameters for getting status
type StatusParams struct {}

// SyncStatesParams represents parameters for syncing states
type SyncStatesParams struct {
    Filter      SyncStateFilterView  `json:"filter"`
    Cursor      *string              `json:"cursor,omitempty"`
    Limit       *string              `json:"limit,omitempty"`
    QueryOption *QueryOptions        `json:"queryOption,omitempty"`
} 