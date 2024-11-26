package bcs

type StructTag struct {
	Address    string
	Module     string
	Name       string
	TypeParams []BcsTypeTag
}

type TypeTag interface{}

type BcsTypeTag struct {
	Bool    *struct{}
	U8      *struct{}
	U16     *struct{}
	U32     *struct{}
	U64     *struct{}
	U128    *struct{}
	U256    *struct{}
	Address *struct{}
	Signer  *struct{}
	Vector  *BcsTypeTag
	Struct  *StructTag
} 