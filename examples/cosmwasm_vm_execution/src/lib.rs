// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, Repy,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    initial_value: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Add { value: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetValue {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ValueResponse {
    value: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MigrateMsg {
    new_value: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CustomSudoMsg {
    UpdateValue { value: u64 },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.storage.set(b"value", &msg.initial_value.to_be_bytes());
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Add { value } => {
            let current_value = deps
                .storage
                .get(b"value")
                .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap_or([0; 8])))
                .unwrap_or(0);
            let new_value = current_value + value;
            deps.storage.set(b"value", &new_value.to_be_bytes());
            Ok(Response::default())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetValue {} => {
            let value = deps
                .storage
                .get(b"value")
                .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap_or([0; 8])))
                .unwrap_or(0);
            to_json_binary(&value)
        }
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    deps.storage.set(b"value", &msg.new_value.to_be_bytes());
    Ok(Response::new().add_attribute("action", "migrate").add_attribute("new_value", msg.new_value.to_string()))
}

#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    Ok(Response::new().add_attribute("action", "reply").add_attribute("id", msg.id.to_string()))
}

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::Custom(sudo_msg) => {
            let custom_msg: CustomSudoMsg = serde_json::from_slice(&sudo_msg)?;
            match custom_msg {
                CustomSudoMsg::UpdateValue { value } => {
                    deps.storage.set(b"value", &value.to_be_bytes());
                    Ok(Response::new().add_attribute("action", "sudo_update_value").add_attribute("new_value", value.to_string()))
                }
            }
        }
        _ => Err(cosmwasm_std::StdError::generic_err("Unsupported sudo message")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg { initial_value: 100 };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let value: u64 =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::GetValue {}).unwrap()).unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn test_execute_add() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let init_msg = InstantiateMsg { initial_value: 100 };
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let exec_msg = ExecuteMsg::Add { value: 50 };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), exec_msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let value: u64 =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::GetValue {}).unwrap()).unwrap();
        assert_eq!(value, 150);
    }

    #[test]
    fn test_query_value() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let init_msg = InstantiateMsg { initial_value: 100 };
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let value: u64 =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::GetValue {}).unwrap()).unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn test_migrate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let init_msg = InstantiateMsg { initial_value: 100 };
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let migrate_msg = MigrateMsg { new_value: 200 };
        let res = migrate(deps.as_mut(), env.clone(), migrate_msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        let value: u64 =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::GetValue {}).unwrap()).unwrap();
        assert_eq!(value, 200);
    }

    #[test]
    fn test_reply() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let reply_msg = Reply { id: 1, result: cosmwasm_std::SubMsgResult::Ok(SubMsg::new("")) };
        let res = reply(deps.as_mut(), env, reply_msg).unwrap();

        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "reply");
        assert_eq!(res.attributes[1].key, "id");
        assert_eq!(res.attributes[1].value, "1");
    }

    #[test]
    fn test_sudo() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let init_msg = InstantiateMsg { initial_value: 100 };
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let sudo_msg = SudoMsg::Custom(to_json_binary(&CustomSudoMsg::UpdateValue { value: 300 }).unwrap());
        let res = sudo(deps.as_mut(), env.clone(), sudo_msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        let value: u64 =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::GetValue {}).unwrap()).unwrap();
        assert_eq!(value, 300);
    }
}
