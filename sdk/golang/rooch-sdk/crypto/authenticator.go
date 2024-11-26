package crypto

import (
	"bytes"
	"encoding/hex"
	"errors"
	"github.com/rooch-network/rooch/sdk/golang/bcs"
	"github.com/rooch-network/rooch/sdk/golang/types"
	"github.com/rooch-network/rooch/sdk/golang/utils"
)

const (
	BitcoinMessagePrefix = "\u0018Bitcoin Signed Message:\n"
	MessageInfoPrefix    = "Rooch Transaction:\n"
)

type BitcoinSignMessage struct {
	messagePrefix string
	messageInfo   string
	txHash       types.Bytes
}

func NewBitcoinSignMessage(txData types.Bytes, messageInfo string) *BitcoinSignMessage {
	msg := messageInfo
	if !strings.HasPrefix(msg, MessageInfoPrefix) {
		msg = MessageInfoPrefix + messageInfo
	}
	
	if !strings.HasSuffix(msg, "\n") {
		msg = msg + "\n"
	}

	return &BitcoinSignMessage{
		messagePrefix: BitcoinMessagePrefix,
		messageInfo:   msg,
		txHash:       txData,
	}
}

func (b *BitcoinSignMessage) Raw() string {
	return b.messageInfo + hex.EncodeToString(b.txHash)
}

func (b *BitcoinSignMessage) Encode() types.Bytes {
	msgHex := []byte(hex.EncodeToString(b.txHash))
	infoBytes := []byte(b.messageInfo)
	prefixBytes := bytes.Join([][]byte{
		[]byte(b.messagePrefix),
		utils.VarintByteNum(len(infoBytes) + len(msgHex)),
	}, []byte{})

	return bytes.Join([][]byte{prefixBytes, infoBytes, msgHex}, []byte{})
}

func (b *BitcoinSignMessage) Hash() types.Bytes {
	return utils.Sha256(b.Encode())
}

type BuiltinAuthValidator int

const (
	ROOCH BuiltinAuthValidator = iota
	BITCOIN
	// ETHEREUM
)

type Authenticator struct {
	authValidatorId int
	payload        types.Bytes
}

func NewAuthenticator(authValidatorId int, payload types.Bytes) *Authenticator {
	return &Authenticator{
		authValidatorId: authValidatorId,
		payload:        payload,
	}
}

func (a *Authenticator) Encode() types.Bytes {
	serialized := bcs.SerializeAuthenticator(&struct {
		AuthValidatorId int
		Payload        types.Bytes
	}{
		AuthValidatorId: a.authValidatorId,
		Payload:        a.payload,
	})
	return serialized
}

func RoochAuth(input types.Bytes, signer Signer) (*Authenticator, error) {
	signature, err := signer.Sign(input)
	if err != nil {
		return nil, err
	}

	pubKeyBytes := signer.GetPublicKey().ToBytes()
	serializedSignature := make([]byte, 1+len(signature)+len(pubKeyBytes))
	serializedSignature[0] = byte(SignatureSchemeToFlag[signer.GetKeyScheme()])
	copy(serializedSignature[1:], signature)
	copy(serializedSignature[1+len(signature):], pubKeyBytes)

	return NewAuthenticator(int(ROOCH), serializedSignature), nil
}

func BitcoinAuth(input *BitcoinSignMessage, signer Signer, signWith string) (*Authenticator, error) {
	if !strings.HasPrefix(input.messageInfo, MessageInfoPrefix) {
		return nil, errors.New("invalid message info")
	}

	messageLength := len([]byte(input.messageInfo)) + len(hex.EncodeToString(input.txHash))
	
	var signData types.Bytes
	if signWith == "hash" {
		signData = input.Hash()
	} else {
		signData = []byte(input.Raw())
	}
 
	signature, err := signer.Sign(signData)
	if err != nil {
		return nil, err
	}

	payload := bcs.SerializeBitcoinAuthPayload(&struct {
		Signature     types.Bytes
		MessagePrefix types.Bytes
		MessageInfo   types.Bytes
		PublicKey     types.Bytes
		FromAddress   types.Bytes
	}{
		Signature:     signature,
		MessagePrefix: bytes.Join([][]byte{
			[]byte(input.messagePrefix),
			utils.VarintByteNum(messageLength),
		}, []byte{}),
		MessageInfo:   []byte(input.messageInfo),
		PublicKey:     signer.GetPublicKey().ToBytes(),
		FromAddress:   []byte(signer.GetBitcoinAddress().ToStr()),
	})

	return NewAuthenticator(int(BITCOIN), payload), nil
} 