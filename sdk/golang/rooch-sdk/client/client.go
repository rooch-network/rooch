package client

import (
	"encoding/hex"
	"errors"
	"math/big"
	
	"github.com/rooch-network/rooch/sdk/golang/crypto"
	"github.com/rooch-network/rooch/sdk/golang/types"
	"github.com/rooch-network/rooch/sdk/golang/address"
	"github.com/rooch-network/rooch/sdk/golang/transactions"
	"github.com/rooch-network/rooch/sdk/golang/session"
)

type RoochClient struct {
	chainID   *big.Int
	transport RoochTransport
}

func NewRoochClient(options RoochClientOptions) *RoochClient {
	var transport RoochTransport
	if options.Transport != nil {
		transport = options.Transport
	} else {
		transport = NewRoochHTTPTransport(options.URL)
	}
	
	return &RoochClient{
		transport: transport,
	}
}

func (c *RoochClient) GetRpcApiVersion() (string, error) {
	var resp struct {
		Info struct {
			Version string `json:"version"`
		} `json:"info"`
	}
	
	err := c.transport.Request("rpc.discover", nil, &resp)
	return resp.Info.Version, err
}

func (c *RoochClient) GetChainId() (*big.Int, error) {
	if c.chainID != nil {
		return c.chainID, nil
	}

	var result string
	err := c.transport.Request("rooch_getChainID", nil, &result)
	if err != nil {
		return nil, err
	}

	chainID, ok := new(big.Int).SetString(result, 10)
	if !ok {
		return nil, errors.New("invalid chain ID format")
	}

	c.chainID = chainID
	return chainID, nil
}

func (c *RoochClient) ExecuteViewFunction(input transactions.CallFunctionArgs) (*types.AnnotatedFunctionResultView, error) {
	callFunction := transactions.NewCallFunction(input)
	
	var result types.AnnotatedFunctionResultView
	err := c.transport.Request("rooch_executeViewFunction", []interface{}{
		map[string]interface{}{
			"function_id": callFunction.FunctionId(),
			"args":        callFunction.EncodeArgs(),
			"ty_args":     callFunction.TypeArgs,
		},
	}, &result)
	
	return &result, err
}

func (c *RoochClient) GetStates(params GetStatesParams) ([]types.ObjectStateView, error) {
	var result []types.ObjectStateView
	err := c.transport.Request("rooch_getStates", []interface{}{
		params.AccessPath,
		params.StateOption,
	}, &result)
	
	if result == nil {
		return []types.ObjectStateView{}, nil
	}
	return result, err
}

func (c *RoochClient) ListStates(params ListStatesParams) (*types.PaginatedStateKVViews, error) {
	var result types.PaginatedStateKVViews
	err := c.transport.Request("rooch_listStates", []interface{}{
		params.AccessPath,
		params.Cursor,
		params.Limit,
		params.StateOption,
	}, &result)
	return &result, err
}

func (c *RoochClient) GetModuleAbi(params GetModuleABIParams) (*types.ModuleABIView, error) {
	var result types.ModuleABIView
	err := c.transport.Request("rooch_getModuleABI", []interface{}{
		params.ModuleAddr,
		params.ModuleName,
	}, &result)
	return &result, err
}

func (c *RoochClient) GetEvents(params GetEventsByEventHandleParams) (*types.PaginatedEventViews, error) {
	var result types.PaginatedEventViews
	err := c.transport.Request("rooch_getEventsByEventHandle", []interface{}{
		params.EventHandleType,
		params.Cursor,
		params.Limit,
		params.DescendingOrder,
		params.EventOptions,
	}, &result)
	return &result, err
}

func (c *RoochClient) QueryEvents(params QueryEventsParams) (*types.PaginatedIndexerEventViews, error) {
	var result types.PaginatedIndexerEventViews
	err := c.transport.Request("rooch_queryEvents", []interface{}{
		params.Filter,
		params.Cursor,
		params.Limit,
		params.QueryOption,
	}, &result)
	return &result, err
}

func (c *RoochClient) QueryInscriptions(params QueryInscriptionsParams) (*types.PaginatedInscriptionStateViews, error) {
	var result types.PaginatedInscriptionStateViews
	err := c.transport.Request("btc_queryInscriptions", []interface{}{
		params.Filter,
		params.Cursor,
		params.Limit,
		params.DescendingOrder,
	}, &result)
	return &result, err
}

func (c *RoochClient) Transfer(params TransferParams) (*types.ExecuteTransactionResponseView, error) {
	tx := transactions.NewTransaction()
	tx.CallFunction(transactions.CallFunctionArgs{
		Target:   "0x3::transfer::transfer_coin",
		Args:     []interface{}{params.Recipient, params.Amount},
		TypeArgs: []string{types.NormalizeTypeArgsToStr(params.CoinType)},
	})

	return c.SignAndExecuteTransaction(struct {
		Transaction interface{}
		Signer     crypto.Signer
		Option     *struct{ WithOutput bool }
	}{
		Transaction: tx,
		Signer:     params.Signer,
	})
}

func (c *RoochClient) TransferObject(params TransferObjectParams) (*types.ExecuteTransactionResponseView, error) {
	tx := transactions.NewTransaction()
	tx.CallFunction(transactions.CallFunctionArgs{
		Target:   "0x3::transfer::transfer_object",
		Args:     []interface{}{params.Recipient, params.ObjectID},
		TypeArgs: []string{types.NormalizeTypeArgsToStr(params.ObjectType)},
	})

	return c.SignAndExecuteTransaction(struct {
		Transaction interface{}
		Signer     crypto.Signer
		Option     *struct{ WithOutput bool }
	}{
		Transaction: tx,
		Signer:     params.Signer,
	})
}

func (c *RoochClient) ResolveBTCAddress(roochAddr string, network address.BitcoinNetworkType) (*address.BitcoinAddress, error) {
	result, err := c.ExecuteViewFunction(transactions.CallFunctionArgs{
		Target: "0x3::address_mapping::resolve_bitcoin",
		Args:   []interface{}{roochAddr},
	})
	if err != nil {
		return nil, err
	}

	if result.VMStatus != "Executed" || len(result.ReturnValues) == 0 {
		return nil, nil
	}

	// Extract address bytes from the result
	value := result.ReturnValues[0].DecodedValue.(map[string]interface{})
	addressBytes := value["value"].(map[string]interface{})["vec"].([]interface{})[0].(map[string]interface{})["value"].(map[string]interface{})["bytes"].(string)

	return address.NewBitcoinAddress(addressBytes, network)
}

func (c *RoochClient) CreateSession(args session.CreateSessionArgs, signer crypto.Signer) (*session.Session, error) {
	return session.Create(session.CreateParams{
		Args:   args,
		Client: c,
		Signer: signer,
	})
}

func (c *RoochClient) RemoveSession(authKey string, signer crypto.Signer) (bool, error) {
	authKeyBytes, err := hex.DecodeString(authKey)
	if err != nil {
		return false, err
	}

	tx := transactions.NewTransaction()
	tx.CallFunction(transactions.CallFunctionArgs{
		Target: "0x3::session_key::remove_session_key_entry",
		Args:   []interface{}{authKeyBytes},
	})

	resp, err := c.SignAndExecuteTransaction(struct {
		Transaction interface{}
		Signer     crypto.Signer
		Option     *struct{ WithOutput bool }
	}{
		Transaction: tx,
		Signer:     signer,
	})
	if err != nil {
		return false, err
	}

	return resp.ExecutionInfo.Status.Type == "executed", nil
}

// ... continuing with more methods ... 