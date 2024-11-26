package bcs

import (
	"testing"
)

func TestStructTagToCanonicalString(t *testing.T) {
	serializer := &Serializer{}

	// Test with no type params
	t.Run("struct tag to string with no type params", func(t *testing.T) {
		testData := StructTag{
			Address: "0x00000000000000000000000000000001",
			Module:  "module1",
			Name:    "name1",
		}
		
		expectStr := "0x0000000000000000000000000000000000000000000000000000000000000001::module1::name1"
		resultStr := serializer.StructTagToCanonicalString(testData)
		
		if resultStr != expectStr {
			t.Errorf("Expected %s, got %s", expectStr, resultStr)
		}
	})

	// Test with type params
	t.Run("struct tag to string with type_params", func(t *testing.T) {
		testData := StructTag{
			Address: "0x00000000000000000000000000000001",
			Module:  "module1",
			Name:    "name1",
			TypeParams: []TypeParam{
				"u8",
				map[string]interface{}{"Vector": "u64"},
				map[string]interface{}{
					"Struct": map[string]interface{}{
						"address": "0x0000000000000000000000000000000a",
						"module":  "module2",
						"name":    "name2",
					},
				},
			},
		}
		
		expectStr := "0x0000000000000000000000000000000000000000000000000000000000000001::module1::name1<u8,vector<u64>,0x000000000000000000000000000000000000000000000000000000000000000a::module2::name2>"
		resultStr := serializer.StructTagToCanonicalString(testData)
		
		if resultStr != expectStr {
			t.Errorf("Expected %s, got %s", expectStr, resultStr)
		}
	})
}

func TestTypeTagToString(t *testing.T) {
	serializer := &Serializer{}

	t.Run("type tag to string with vector type", func(t *testing.T) {
		testData := TypeTag{"Vector": "u64"}
		expectStr := "vector<u64>"
		resultStr, err := serializer.TypeTagToString(testData)
		
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if resultStr != expectStr {
			t.Errorf("Expected %s, got %s", expectStr, resultStr)
		}
	})

	t.Run("type tag to string with struct type", func(t *testing.T) {
		testData := TypeTag{
			"Struct": map[string]interface{}{
				"address": "0x0000000000000000000000000000000a",
				"module":  "module2",
				"name":    "name2",
			},
		}
		expectStr := "0x000000000000000000000000000000000000000000000000000000000000000a::module2::name2"
		resultStr, err := serializer.TypeTagToString(testData)
		
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if resultStr != expectStr {
			t.Errorf("Expected %s, got %s", expectStr, resultStr)
		}
	})
}

func TestStructTagToObjectID(t *testing.T) {
	serializer := &Serializer{}

	t.Run("test named object id", func(t *testing.T) {
		testData := StructTag{
			Address:    "0x2",
			Module:     "timestamp",
			Name:       "Timestamp",
			TypeParams: []TypeParam{},
		}
		
		expectObjectID := "0x3a7dfe7a9a5cd608810b5ebd60c7adf7316667b17ad5ae703af301b74310bcca"
		resultObjectID := serializer.StructTagToObjectID(testData)
		
		if resultObjectID != expectObjectID {
			t.Errorf("Expected %s, got %s", expectObjectID, resultObjectID)
		}
	})

	t.Run("test_account_named_object_id", func(t *testing.T) {
		testData := StructTag{
			Address: "0x3",
			Module:  "coin_store",
			Name:    "CoinStore",
			TypeParams: []TypeParam{
				map[string]interface{}{
					"Struct": map[string]interface{}{
						"address":    "0x3",
						"module":     "gas_coin",
						"name":       "RGas",
						"typeParams": []interface{}{},
					},
				},
			},
		}
		
		expectObjectID := "0xfdda11f9cc18bb30973779eb3610329d7e0e3c6ecce05b4d77b5a839063bff66"
		resultObjectID := serializer.StructTagToObjectID(testData)
		
		if resultObjectID != expectObjectID {
			t.Errorf("Expected %s, got %s", expectObjectID, resultObjectID)
		}
	})
} 