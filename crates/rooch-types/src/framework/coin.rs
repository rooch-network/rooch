// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::move_std::option::MoveOption;
use moveos_types::move_std::string::MoveString;
use moveos_types::move_types;
use moveos_types::moveos_std::object::{self, ObjectID};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType, PlaceholderStruct};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("coin");

pub const DEFAULT_DECIMALS: u8 = 9;

/// Rust bindings for RoochFramework coin module
pub struct CoinModule<'a> {
    //avoid #[warn(dead_code)] warning
    //TODO change this to private after we use the caller
    pub caller: &'a dyn MoveFunctionCaller,
}

impl<'a> CoinModule<'a> {
    pub fn coin_info_id(coin_type: StructTag) -> ObjectID {
        let coin_info_struct_tag =
            CoinInfo::<PlaceholderStruct>::struct_tag_with_coin_type(coin_type);
        object::named_object_id(&coin_info_struct_tag)
    }
}

impl<'a> ModuleBinding<'a> for CoinModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin<T> {
    pub value: U256,
    pub phantom: std::marker::PhantomData<T>,
}

impl<T> Coin<T> {
    pub fn new(value: U256) -> Self {
        Coin {
            value,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> MoveStructType for Coin<T>
where
    T: MoveStructType,
{
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Coin");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![T::struct_tag().into()],
        }
    }
}

impl<T> MoveStructState for Coin<T>
where
    T: MoveStructType,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U256,
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo<CoinType> {
    coin_type: MoveString,
    name: MoveString,
    symbol: MoveString,
    icon_url: MoveOption<MoveString>,
    decimals: u8,
    supply: U256,
    phantom: std::marker::PhantomData<CoinType>,
}

impl<CoinType> MoveStructType for CoinInfo<CoinType>
where
    CoinType: MoveStructType,
{
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinInfo");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![CoinType::struct_tag().into()],
        }
    }
}

impl<CoinType> MoveStructState for CoinInfo<CoinType>
where
    CoinType: MoveStructType,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            MoveString::type_layout(),
            MoveString::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            move_core_types::value::MoveTypeLayout::U8,
            move_core_types::value::MoveTypeLayout::U256,
        ])
    }
}

impl<CoinType> CoinInfo<CoinType>
where
    CoinType: MoveStructType,
{
    pub fn struct_tag_with_coin_type(coin_type: StructTag) -> StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![coin_type.into()],
        }
    }
}

/// The StructTag for the InvalidCoinType error
static INVALID_COIN_TYPE: Lazy<StructTag> = Lazy::new(|| StructTag {
    address: ROOCH_FRAMEWORK_ADDRESS,
    module: MODULE_NAME.to_owned(),
    name: ident_str!("InvalidCoinType").to_owned(),
    type_params: vec![],
});

impl<CoinType> CoinInfo<CoinType> {
    pub fn coin_type(&self) -> String {
        self.coin_type.to_string()
    }
    pub fn coin_type_tag(&self) -> StructTag {
        //Because the coin_type is a canonical string, we can parse it to a StructTag
        //For avoid panic, we use unwrap_or to return InvalidCoinType if the parsing failed
        move_types::parse_struct_tag(&self.coin_type.to_string())
            .unwrap_or(INVALID_COIN_TYPE.clone())
    }
    pub fn name(&self) -> String {
        self.name.to_string()
    }
    pub fn symbol(&self) -> String {
        self.symbol.to_string()
    }
    pub fn icon_url(&self) -> Option<String> {
        self.icon_url.clone().map(|v| v.to_string()).into()
    }
    pub fn decimals(&self) -> u8 {
        self.decimals
    }
    pub fn supply(&self) -> U256 {
        self.supply
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hex::DisplayHex;
    use fastcrypto::encoding::{Base64, Encoding};

    #[test]
    pub fn test_gas_coin_icon_url() {
        let base64_data = "PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz4KPHN2ZyBpZD0idXVpZC1mM2MxMGRhMy05NDE3LTQxMGUtYTNhYi04Y2UxYWI3ZDc1YTIiIGRhdGEtbmFtZT0i5Zu+5bGCIDEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUwMCA1MDAiPgogIDxkZWZzPgogICAgPHN0eWxlPgogICAgICAudXVpZC1mOGVkMWE5MS1kNzcwLTQ0ZTQtYjlhMS04ODk4YWVkNzllMjMgewogICAgICAgIGZpbGw6ICMwMDY4NDA7CiAgICAgIH0KCiAgICAgIC51dWlkLTA2ZTJkZWJmLTBhZWYtNDk0ZC1iNTc3LTZkNDk2MTFlMWNmOCB7CiAgICAgICAgZmlsbDogI2IyZmYwNDsKICAgICAgfQogICAgPC9zdHlsZT4KICA8L2RlZnM+CiAgPGNpcmNsZSBjbGFzcz0idXVpZC0wNmUyZGViZi0wYWVmLTQ5NGQtYjU3Ny02ZDQ5NjExZTFjZjgiIGN4PSIyNTAiIGN5PSIyNTAiIHI9IjI1MCIvPgogIDxwYXRoIGNsYXNzPSJ1dWlkLWY4ZWQxYTkxLWQ3NzAtNDRlNC1iOWExLTg4OThhZWQ3OWUyMyIgZD0iTTM0Ni4zOSw0MDMuMTVjLTI2LjE3LTIzLjE4LTUxLjU4LTQ1LjY5LTc3LjczLTY4Ljg2LS4xMywyLjU4LS4yOCw0LjE4LS4yOSw1Ljc3LS4xOSwzMC4yNC0uMzYsNjAuNDgtLjU0LDkwLjczcS0uMDQsNi40OC02Ljc0LDYuNDVjLTguNzgtLjA1LTE3LjU3LS4yNy0yNi4zNS0uMDctMy40NiwuMDgtNC4zNi0uODMtNC4zMi00LjMsLjMzLTMxLjYyLC40Ni02My4yMywuNjQtOTQuODUsMC0xLjA1LC4wMS0yLjA5LC4wMi00LjEyLTI2LjQ2LDIyLjk5LTUyLjI5LDQ1LjQ0LTc4LjA2LDY3Ljg0LTEuNi0uNzItLjk2LTEuOTUtLjk2LTIuODgsLjA0LTE0Ljk3LC4yLTI5Ljk0LC4xNy00NC45LDAtMi4zNiwuNzUtMy44NiwyLjUyLTUuNDIsMjguOTktMjUuNTcsNTcuOTEtNTEuMjIsODYuODUtNzYuODQsLjY3LS41OSwxLjQ1LTEuMDcsMi40LTEuNzYtMS44OS0xLjY2LTMuNjQtMS4wOS01LjE4LTEuMS0yNy4xMi0uMi01NC4yNC0uNDMtODEuMzYtLjQxLTMuOTEsMC00Ljg4LTEuMDUtNC43NC00LjgzLC4zNC05LjQ2LC40Mi0xOC45NCwuMTYtMjguNC0uMTEtMy44MywxLjI5LTQuMzksNC42OC00LjM1LDI2Ljk2LC4yOCw1My45MywuMzgsODAuOSwuNTQsMS42OCwwLDMuMzcsLjAyLDUuODMsLjAzLTIuMDgtMy41OS01LjA2LTUuNS03LjUzLTcuNzQtMjctMjQuNTYtNTQuMDYtNDkuMDUtODEuMTctNzMuNDktMS40OS0xLjM1LTIuMTUtMi42NC0yLjEzLTQuNjYsLjE3LTE2LjAzLC4yMy0zMi4wNywuMzMtNDkuMjQsMjYuMjQsMjMuMzQsNTEuODYsNDYuMTIsNzguMjcsNjkuNjEsLjAxLTIuNDYsLjAyLTQuMDYsLjAzLTUuNjUsLjE4LTMxLjAxLC40My02Mi4wMSwuNDUtOTMuMDIsMC0zLjcxLDEuMS00LjU1LDQuNjUtNC40NCw5LjM5LC4yOSwxOC44LC40MiwyOC4xOSwuMTQsNC4wNi0uMTIsNC42OSwxLjI4LDQuNjUsNC45My0uMzMsMzAuNy0uNDQsNjEuNC0uNjIsOTIuMSwwLDEuNjMtLjAyLDMuMjctLjA0LDYuMTksMjYuNTItMjMuMTMsNTIuMTYtNDUuNSw3Ny45OC02OC4wMSwxLjA0LDEuNiwuNjQsMi45OSwuNjQsNC4yNy0uMDUsMTMuOS0uMjksMjcuOC0uMTMsNDEuNywuMDUsMy44Ni0xLjE0LDYuNDEtNC4wNSw4Ljk3LTI4LjMxLDI0Ljk1LTU2LjQ4LDUwLjA1LTg0LjY5LDc1LjEtLjc4LC42OS0xLjUyLDEuNDEtMi40NSwyLjI3LDEuNjYsMS43MSwzLjYsMS4wMyw1LjI3LDEuMDQsMjYuOTYsLjIxLDUzLjkzLC40MSw4MC45LC40NCwzLjQyLDAsNC43NywuNiw0LjYyLDQuNDEtLjM3LDkuNjEtLjM3LDE5LjI0LS4xOSwyOC44NiwuMDYsMy4zOC0uNzcsNC4zNS00LjMxLDQuMzEtMjcuMTItLjMyLTU0LjI0LS40LTgxLjM2LS41Ni0xLjY0LDAtMy4yOC0uMDItNi4xNC0uMDQsOC43Nyw3Ljk0LDE2LjY0LDE1LjA5LDI0LjU0LDIyLjIyLDIxLjYzLDE5LjUyLDQzLjI2LDM5LjA0LDY0LjkxLDU4LjUzLDEuMDgsLjk3LDEuODEsMS44OSwxLjgsMy40Ni0uMTQsMTYuMzItLjIyLDMyLjY0LS4zMiw1MC4wNVoiLz4KPC9zdmc+";
        let bytes_data = Base64::decode(base64_data).unwrap();
        let hex_data = bytes_data.to_lower_hex_string();

        let rgas_icon_hex = "3c3f786d6c2076657273696f6e3d22312e302220656e636f64696e673d225554462d38223f3e0a3c7376672069643d22757569642d66336331306461332d393431372d343130652d613361622d3863653161623764373561322220646174612d6e616d653d22e59bbee5b18220312220786d6c6e733d22687474703a2f2f7777772e77332e6f72672f323030302f737667222076696577426f783d223020302035303020353030223e0a20203c646566733e0a202020203c7374796c653e0a2020202020202e757569642d66386564316139312d643737302d343465342d623961312d383839386165643739653233207b0a202020202020202066696c6c3a20233030363834303b0a2020202020207d0a0a2020202020202e757569642d30366532646562662d306165662d343934642d623537372d366434393631316531636638207b0a202020202020202066696c6c3a20236232666630343b0a2020202020207d0a202020203c2f7374796c653e0a20203c2f646566733e0a20203c636972636c6520636c6173733d22757569642d30366532646562662d306165662d343934642d623537372d366434393631316531636638222063783d22323530222063793d223235302220723d22323530222f3e0a20203c7061746820636c6173733d22757569642d66386564316139312d643737302d343465342d623961312d3838393861656437396532332220643d224d3334362e33392c3430332e3135632d32362e31372d32332e31382d35312e35382d34352e36392d37372e37332d36382e38362d2e31332c322e35382d2e32382c342e31382d2e32392c352e37372d2e31392c33302e32342d2e33362c36302e34382d2e35342c39302e3733712d2e30342c362e34382d362e37342c362e3435632d382e37382d2e30352d31372e35372d2e32372d32362e33352d2e30372d332e34362c2e30382d342e33362d2e38332d342e33322d342e332c2e33332d33312e36322c2e34362d36332e32332c2e36342d39342e38352c302d312e30352c2e30312d322e30392c2e30322d342e31322d32362e34362c32322e39392d35322e32392c34352e34342d37382e30362c36372e38342d312e362d2e37322d2e39362d312e39352d2e39362d322e38382c2e30342d31342e39372c2e322d32392e39342c2e31372d34342e392c302d322e33362c2e37352d332e38362c322e35322d352e34322c32382e39392d32352e35372c35372e39312d35312e32322c38362e38352d37362e38342c2e36372d2e35392c312e34352d312e30372c322e342d312e37362d312e38392d312e36362d332e36342d312e30392d352e31382d312e312d32372e31322d2e322d35342e32342d2e34332d38312e33362d2e34312d332e39312c302d342e38382d312e30352d342e37342d342e38332c2e33342d392e34362c2e34322d31382e39342c2e31362d32382e342d2e31312d332e38332c312e32392d342e33392c342e36382d342e33352c32362e39362c2e32382c35332e39332c2e33382c38302e392c2e35342c312e36382c302c332e33372c2e30322c352e38332c2e30332d322e30382d332e35392d352e30362d352e352d372e35332d372e37342d32372d32342e35362d35342e30362d34392e30352d38312e31372d37332e34392d312e34392d312e33352d322e31352d322e36342d322e31332d342e36362c2e31372d31362e30332c2e32332d33322e30372c2e33332d34392e32342c32362e32342c32332e33342c35312e38362c34362e31322c37382e32372c36392e36312c2e30312d322e34362c2e30322d342e30362c2e30332d352e36352c2e31382d33312e30312c2e34332d36322e30312c2e34352d39332e30322c302d332e37312c312e312d342e35352c342e36352d342e34342c392e33392c2e32392c31382e382c2e34322c32382e31392c2e31342c342e30362d2e31322c342e36392c312e32382c342e36352c342e39332d2e33332c33302e372d2e34342c36312e342d2e36322c39322e312c302c312e36332d2e30322c332e32372d2e30342c362e31392c32362e35322d32332e31332c35322e31362d34352e352c37372e39382d36382e30312c312e30342c312e362c2e36342c322e39392c2e36342c342e32372d2e30352c31332e392d2e32392c32372e382d2e31332c34312e372c2e30352c332e38362d312e31342c362e34312d342e30352c382e39372d32382e33312c32342e39352d35362e34382c35302e30352d38342e36392c37352e312d2e37382c2e36392d312e35322c312e34312d322e34352c322e32372c312e36362c312e37312c332e362c312e30332c352e32372c312e30342c32362e39362c2e32312c35332e39332c2e34312c38302e392c2e34342c332e34322c302c342e37372c2e362c342e36322c342e34312d2e33372c392e36312d2e33372c31392e32342d2e31392c32382e38362c2e30362c332e33382d2e37372c342e33352d342e33312c342e33312d32372e31322d2e33322d35342e32342d2e342d38312e33362d2e35362d312e36342c302d332e32382d2e30322d362e31342d2e30342c382e37372c372e39342c31362e36342c31352e30392c32342e35342c32322e32322c32312e36332c31392e35322c34332e32362c33392e30342c36342e39312c35382e35332c312e30382c2e39372c312e38312c312e38392c312e382c332e34362d2e31342c31362e33322d2e32322c33322e36342d2e33322c35302e30355a222f3e0a3c2f7376673e";
        assert_eq!(rgas_icon_hex, hex_data);
    }
}
