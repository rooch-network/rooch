package e2e

import (
	"encoding/json"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

type TestBox struct {
	// Add necessary fields similar to JS TestBox
	bitcoinContainer *Container
	ordContainer    *Container
	client          *RoochClient
	keypair         *Keypair
}

func TestBitcoinAssetsAPI(t *testing.T) {
	var testBox TestBox

	// Setup - equivalent to beforeAll
	t.Run("setup", func(t *testing.T) {
		testBox = TestBox.Setup()
		require.NoError(t, testBox.LoadBitcoinEnv())
		require.NoError(t, testBox.LoadORDEnv())
		require.NoError(t, testBox.LoadRoochEnv("local", 0))
	})

	// Cleanup - equivalent to afterAll
	defer func() {
		testBox.CleanEnv()
	}()

	// Test query UTXO
	t.Run("query utxo should be success", func(t *testing.T) {
		addr := testBox.keypair.GetSchnorrPublicKey().ToAddress().BitcoinAddress.ToStr()

		result, err := testBox.bitcoinContainer.ExecuteRpcCommandRaw([]string{}, "generatetoaddress", []string{
			"50",
			addr,
		})
		require.NoError(t, err)
		require.NotNil(t, result)

		// Wait for rooch indexer
		time.Sleep(10 * time.Second)

		utxos, err := testBox.client.QueryUTXO(&QueryFilter{
			Owner: addr,
		})
		require.NoError(t, err)
		require.Greater(t, len(utxos.Data), 0)
	})

	// Test query inscriptions
	t.Run("query inscriptions should be success", func(t *testing.T) {
		// Init wallet
		result, err := testBox.ordContainer.ExecCmd("wallet create")
		require.NoError(t, err)
		require.Equal(t, 0, result.ExitCode)

		result, err = testBox.ordContainer.ExecCmd("wallet receive")
		require.NoError(t, err)
		require.Equal(t, 0, result.ExitCode)

		var receiveResp struct {
			Addresses []string `json:"addresses"`
		}
		err = json.Unmarshal([]byte(result.Output), &receiveResp)
		require.NoError(t, err)
		addr := receiveResp.Addresses[0]

		// Mint UTXO
		result, err = testBox.bitcoinContainer.ExecuteRpcCommandRaw([]string{}, "generatetoaddress", []string{
			"101",
			addr,
		})
		require.NoError(t, err)
		require.NotNil(t, result)

		// Wait for ord sync and index
		time.Sleep(10 * time.Second)

		result, err = testBox.ordContainer.ExecCmd("wallet balance")
		require.NoError(t, err)
		
		var balanceResp struct {
			Total int64 `json:"total"`
		}
		err = json.Unmarshal([]byte(result.Output), &balanceResp)
		require.NoError(t, err)
		require.Equal(t, int64(5000000000), balanceResp.Total)

		// Create inscription
		inscriptionContent := `{"p":"brc-20","op":"mint","tick":"Rooch","amt":"1"}`
		filePath := fmt.Sprintf("/%s/hello.txt", testBox.ordContainer.GetHostDataPath())
		err = os.WriteFile(filePath, []byte(inscriptionContent), 0644)
		require.NoError(t, err)

		result, err = testBox.ordContainer.ExecCmd(fmt.Sprintf("wallet inscribe --fee-rate 1 --file /data/hello.txt --destination %s", addr))
		require.NoError(t, err)
		require.Equal(t, 0, result.ExitCode)

		// Mint UTXO
		result, err = testBox.bitcoinContainer.ExecuteRpcCommandRaw([]string{}, "generatetoaddress", []string{
			"1",
			addr,
		})
		require.NoError(t, err)
		require.NotNil(t, result)

		// Wait for rooch indexer
		time.Sleep(10 * time.Second)

		utxos, err := testBox.client.QueryUTXO(&QueryFilter{
			Owner: addr,
		})
		require.NoError(t, err)
		require.Greater(t, len(utxos.Data), 0)

		inscriptions, err := testBox.client.QueryInscriptions(&QueryFilter{
			Owner: addr,
		})
		require.NoError(t, err)
		require.Greater(t, len(inscriptions.Data), 0)
	})
} 