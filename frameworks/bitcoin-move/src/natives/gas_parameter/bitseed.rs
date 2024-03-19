use crate::natives::ord::bitseed::ArgsPackingGasParameters;
use rooch_framework::natives::gas_parameter::native::MUL;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(ArgsPackingGasParameters, "bitseed", [
    [.base, "from_witness.base", 100 * MUL],
    [.per_byte, "from_witness.base", 1000 * MUL],
]);