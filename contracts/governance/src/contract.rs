use crate::msg::HandleMsg::Signup;
use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{
    owner_cfg_read, owner_cfg_store, payments_delete, payments_read, payments_store, Config,
};
use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use name_service::msg::HandleMsg::{Deregister, Register};
use name_service::msg::{InitHook, NSInitMsg};
use name_service::state::Name;

const LUNA: &str = "uluna";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        owner: env.message.sender,
        name_service_address: Default::default(),
    };
    owner_cfg_store(&mut deps.storage).save(&config).unwrap();
    let response = InitResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.nameservice_code_id,
            msg: to_binary(&NSInitMsg {
                hook: Some(InitHook {
                    contract_addr: env.contract.address,
                    msg: to_binary(&Signup {}).unwrap(),
                }),
            })?,
            send: vec![],
            label: None,
        })],
        log: vec![],
    };
    Ok(response)
}