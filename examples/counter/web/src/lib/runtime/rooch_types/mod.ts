
import { Serializer, Deserializer } from '../serde/mod.ts';
import { BcsSerializer, BcsDeserializer } from '../bcs/mod.ts';
import { Optional, Seq, Tuple, ListTuple, unit, bool, int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, char, str, bytes } from '../serde/mod.ts';

export class AccountAddress {

constructor (public value: ListTuple<[uint8]>) {
}

public serialize(serializer: Serializer): void {
  Helpers.serializeArray32U8Array(this.value, serializer);
}

static deserialize(deserializer: Deserializer): AccountAddress {
  const value = Helpers.deserializeArray32U8Array(deserializer);
  return new AccountAddress(value);
}

}
export class Bundle {

constructor (public value: Seq<bytes>) {
}

public serialize(serializer: Serializer): void {
  Helpers.serializeVectorBytes(this.value, serializer);
}

static deserialize(deserializer: Deserializer): Bundle {
  const value = Helpers.deserializeVectorBytes(deserializer);
  return new Bundle(value);
}

}
export class FunctionCall {

constructor (public function_id: Identifier, public ty_args: Seq<TypeTag>, public args: Seq<bytes>) {
}

public serialize(serializer: Serializer): void {
  this.function_id.serialize(serializer);
  Helpers.serializeVectorTypeTag(this.ty_args, serializer);
  Helpers.serializeVectorBytes(this.args, serializer);
}

static deserialize(deserializer: Deserializer): FunctionCall {
  const function_id = Identifier.deserialize(deserializer);
  const ty_args = Helpers.deserializeVectorTypeTag(deserializer);
  const args = Helpers.deserializeVectorBytes(deserializer);
  return new FunctionCall(function_id,ty_args,args);
}

}
export class Identifier {

constructor (public value: str) {
}

public serialize(serializer: Serializer): void {
  serializer.serializeStr(this.value);
}

static deserialize(deserializer: Deserializer): Identifier {
  const value = deserializer.deserializeStr();
  return new Identifier(value);
}

}
export class ModuleId {

constructor (public address: AccountAddress, public name: Identifier) {
}

public serialize(serializer: Serializer): void {
  this.address.serialize(serializer);
  this.name.serialize(serializer);
}

static deserialize(deserializer: Deserializer): ModuleId {
  const address = AccountAddress.deserialize(deserializer);
  const name = Identifier.deserialize(deserializer);
  return new ModuleId(address,name);
}

}
export abstract class MoveAction {
abstract serialize(serializer: Serializer): void;

static deserialize(deserializer: Deserializer): MoveAction {
  const index = deserializer.deserializeVariantIndex();
  switch (index) {
    case 0: return MoveActionVariantScript.load(deserializer);
    case 1: return MoveActionVariantFunction.load(deserializer);
    case 2: return MoveActionVariantModuleBundle.load(deserializer);
    default: throw new Error("Unknown variant index for MoveAction: " + index);
  }
}
}


export class MoveActionVariantScript extends MoveAction {

constructor (public value: ScriptCall) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(0);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): MoveActionVariantScript {
  const value = ScriptCall.deserialize(deserializer);
  return new MoveActionVariantScript(value);
}

}

export class MoveActionVariantFunction extends MoveAction {

constructor (public value: FunctionCall) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(1);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): MoveActionVariantFunction {
  const value = FunctionCall.deserialize(deserializer);
  return new MoveActionVariantFunction(value);
}

}

export class MoveActionVariantModuleBundle extends MoveAction {

constructor (public value: Bundle) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(2);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): MoveActionVariantModuleBundle {
  const value = Bundle.deserialize(deserializer);
  return new MoveActionVariantModuleBundle(value);
}

}
export class ScriptCall {

constructor (public code: bytes, public ty_args: Seq<TypeTag>, public args: Seq<bytes>) {
}

public serialize(serializer: Serializer): void {
  serializer.serializeBytes(this.code);
  Helpers.serializeVectorTypeTag(this.ty_args, serializer);
  Helpers.serializeVectorBytes(this.args, serializer);
}

static deserialize(deserializer: Deserializer): ScriptCall {
  const code = deserializer.deserializeBytes();
  const ty_args = Helpers.deserializeVectorTypeTag(deserializer);
  const args = Helpers.deserializeVectorBytes(deserializer);
  return new ScriptCall(code,ty_args,args);
}

}
export class StructTag {

constructor (public address: AccountAddress, public module: Identifier, public name: Identifier, public type_args: Seq<TypeTag>) {
}

public serialize(serializer: Serializer): void {
  this.address.serialize(serializer);
  this.module.serialize(serializer);
  this.name.serialize(serializer);
  Helpers.serializeVectorTypeTag(this.type_args, serializer);
}

static deserialize(deserializer: Deserializer): StructTag {
  const address = AccountAddress.deserialize(deserializer);
  const module = Identifier.deserialize(deserializer);
  const name = Identifier.deserialize(deserializer);
  const type_args = Helpers.deserializeVectorTypeTag(deserializer);
  return new StructTag(address,module,name,type_args);
}

}
export abstract class TypeTag {
abstract serialize(serializer: Serializer): void;

static deserialize(deserializer: Deserializer): TypeTag {
  const index = deserializer.deserializeVariantIndex();
  switch (index) {
    case 0: return TypeTagVariantbool.load(deserializer);
    case 1: return TypeTagVariantu8.load(deserializer);
    case 2: return TypeTagVariantu64.load(deserializer);
    case 3: return TypeTagVariantu128.load(deserializer);
    case 4: return TypeTagVariantaddress.load(deserializer);
    case 5: return TypeTagVariantsigner.load(deserializer);
    case 6: return TypeTagVariantvector.load(deserializer);
    case 7: return TypeTagVariantstruct.load(deserializer);
    case 8: return TypeTagVariantu16.load(deserializer);
    case 9: return TypeTagVariantu32.load(deserializer);
    case 10: return TypeTagVariantu256.load(deserializer);
    default: throw new Error("Unknown variant index for TypeTag: " + index);
  }
}
}


export class TypeTagVariantbool extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(0);
}

static load(deserializer: Deserializer): TypeTagVariantbool {
  return new TypeTagVariantbool();
}

}

export class TypeTagVariantu8 extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(1);
}

static load(deserializer: Deserializer): TypeTagVariantu8 {
  return new TypeTagVariantu8();
}

}

export class TypeTagVariantu64 extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(2);
}

static load(deserializer: Deserializer): TypeTagVariantu64 {
  return new TypeTagVariantu64();
}

}

export class TypeTagVariantu128 extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(3);
}

static load(deserializer: Deserializer): TypeTagVariantu128 {
  return new TypeTagVariantu128();
}

}

export class TypeTagVariantaddress extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(4);
}

static load(deserializer: Deserializer): TypeTagVariantaddress {
  return new TypeTagVariantaddress();
}

}

export class TypeTagVariantsigner extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(5);
}

static load(deserializer: Deserializer): TypeTagVariantsigner {
  return new TypeTagVariantsigner();
}

}

export class TypeTagVariantvector extends TypeTag {

constructor (public value: TypeTag) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(6);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): TypeTagVariantvector {
  const value = TypeTag.deserialize(deserializer);
  return new TypeTagVariantvector(value);
}

}

export class TypeTagVariantstruct extends TypeTag {

constructor (public value: StructTag) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(7);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): TypeTagVariantstruct {
  const value = StructTag.deserialize(deserializer);
  return new TypeTagVariantstruct(value);
}

}

export class TypeTagVariantu16 extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(8);
}

static load(deserializer: Deserializer): TypeTagVariantu16 {
  return new TypeTagVariantu16();
}

}

export class TypeTagVariantu32 extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(9);
}

static load(deserializer: Deserializer): TypeTagVariantu32 {
  return new TypeTagVariantu32();
}

}

export class TypeTagVariantu256 extends TypeTag {
constructor () {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(10);
}

static load(deserializer: Deserializer): TypeTagVariantu256 {
  return new TypeTagVariantu256();
}

}
export class Helpers {
  static serializeArray32U8Array(value: ListTuple<[uint8]>, serializer: Serializer): void {
    value.forEach((item) =>{
        serializer.serializeU8(item[0]);
    });
  }

  static deserializeArray32U8Array(deserializer: Deserializer): ListTuple<[uint8]> {
    const list: ListTuple<[uint8]> = [];
    for (let i = 0; i < 32; i++) {
        list.push([deserializer.deserializeU8()]);
    }
    return list;
  }

  static serializeVectorTypeTag(value: Seq<TypeTag>, serializer: Serializer): void {
    serializer.serializeLen(value.length);
    value.forEach((item: TypeTag) => {
        item.serialize(serializer);
    });
  }

  static deserializeVectorTypeTag(deserializer: Deserializer): Seq<TypeTag> {
    const length = deserializer.deserializeLen();
    const list: Seq<TypeTag> = [];
    for (let i = 0; i < length; i++) {
        list.push(TypeTag.deserialize(deserializer));
    }
    return list;
  }

  static serializeVectorBytes(value: Seq<bytes>, serializer: Serializer): void {
    serializer.serializeLen(value.length);
    value.forEach((item: bytes) => {
        serializer.serializeBytes(item);
    });
  }

  static deserializeVectorBytes(deserializer: Deserializer): Seq<bytes> {
    const length = deserializer.deserializeLen();
    const list: Seq<bytes> = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializer.deserializeBytes());
    }
    return list;
  }

}

