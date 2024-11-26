package crypto

import (
	"testing"
	"bytes"
)

func TestBitcoinSignMessage(t *testing.T) {
	t.Run("should correctly construct with valid txData and messageInfo", func(t *testing.T) {
		txData := []byte{1, 2, 3, 4}
		messageInfo := "Test Message Info"
		bitcoinSignMessage := NewBitcoinSignMessage(txData, messageInfo)

		if bitcoinSignMessage.MessagePrefix != "\u0018Bitcoin Signed Message:\n" {
			t.Errorf("Expected message prefix to be '\\u0018Bitcoin Signed Message:\\n', got %s", bitcoinSignMessage.MessagePrefix)
		}

		expectedMessageInfo := "Rooch Transaction:\nTest Message Info\n"
		if bitcoinSignMessage.MessageInfo != expectedMessageInfo {
			t.Errorf("Expected message info to be '%s', got %s", expectedMessageInfo, bitcoinSignMessage.MessageInfo)
		}

		if !bytes.Equal(bitcoinSignMessage.TxHash, txData) {
			t.Errorf("Expected txHash to equal input txData")
		}
	})

	t.Run("should correctly generate raw message string", func(t *testing.T) {
		txData := []byte{1, 2, 3, 4}
		messageInfo := "Test Message Info"
		bitcoinSignMessage := NewBitcoinSignMessage(txData, messageInfo)

		if !bytes.Equal(bitcoinSignMessage.TxHash, txData) {
			t.Errorf("Expected txHash to equal input txData")
		}

		expected := "Rooch Transaction:\nTest Message Info\n01020304"
		if bitcoinSignMessage.Raw() != expected {
			t.Errorf("Expected raw message to be '%s', got %s", expected, bitcoinSignMessage.Raw())
		}
	})

	t.Run("should handle empty messageInfo gracefully", func(t *testing.T) {
		txData := []byte{}
		messageInfo := ""
		bitcoinSignMessage := NewBitcoinSignMessage(txData, messageInfo)

		expectedMessageInfo := "Rooch Transaction:\n"
		if bitcoinSignMessage.MessageInfo != expectedMessageInfo {
			t.Errorf("Expected message info to be '%s', got %s", expectedMessageInfo, bitcoinSignMessage.MessageInfo)
		}

		if bitcoinSignMessage.Raw() != expectedMessageInfo {
			t.Errorf("Expected raw message to be '%s', got %s", expectedMessageInfo, bitcoinSignMessage.Raw())
		}
	})

	t.Run("should correctly encode message with valid txHash and messageInfo", func(t *testing.T) {
		txData := []byte{0x01, 0x02, 0x03, 0x04}
		messageInfo := "Example message info"
		bitcoinSignMessage := NewBitcoinSignMessage(txData, messageInfo)
		encodedData := bitcoinSignMessage.Encode()

		if len(encodedData) == 0 {
			t.Error("Expected encoded data to not be empty")
		}

		if len(encodedData) > 255 {
			t.Error("Expected encoded data length to be less than or equal to 255")
		}
	})
} 