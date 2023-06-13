import { BcsSerializer } from "../lib/runtime/bcs/bcsSerializer"

function serializeMoveAction(action: MoveAction): Uint8Array {
    const serializer = new BcsSerializer();
    
    Object.entries(action).forEach(([key, value]) => {
        switch (key) {
            case "Script":
                serializer.serializeVariantIndex(0);
                serializer.serializeStruct(value as ScriptCall);
                break;
            case "Function":
                serializer.serialize_variant_index(1);
                serializer.serialize_struct(value as FunctionCall);
                break;
            case "ModuleBundle":
                serializer.serialize_variant_index(2);
                serializer.serialize_len((value as Uint8Array[]).length);
                (value as Uint8Array[]).forEach((item) => serializer.serialize_bytes(item));
                break;
            default:
                throw new Error("Invalid MoveAction type");
        }
    });

    return serializer.getBytes();
}
