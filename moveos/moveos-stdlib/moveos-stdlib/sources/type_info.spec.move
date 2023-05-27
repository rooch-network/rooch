spec moveos_std::type_info {

    spec native fun spec_is_struct<T>(): bool;

    spec type_of<T>(): TypeInfo {
        // Move Prover natively supports this function.
        // This function will abort if `T` is not a struct type.
    }

    spec type_name<T>(): string::String {
        // Move Prover natively supports this function.
    }

    // The chain ID is modeled as an uninterpreted function.
    spec fun spec_chain_id_internal(): u8;
}
