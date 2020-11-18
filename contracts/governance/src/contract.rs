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

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Subscribe { name } => handle_subscribe(deps, env, name),
        HandleMsg::Unsubscribe { name } => handle_unsubscribe(deps, env, name),
        HandleMsg::Signup {} => handle_signup(deps, env),
    }
}

pub fn handle_signup<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut config = owner_cfg_read(&deps.storage).load().unwrap();
    config.name_service_address = env.message.sender;
    owner_cfg_store(&mut deps.storage).save(&config)?;
    Ok(Default::default())
}

pub fn handle_subscribe<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
) -> StdResult<HandleResponse> {
    let sent_value = env.message.sent_funds.get(0);
    if sent_value.is_none()
        || sent_value.unwrap().denom != LUNA
        || sent_value.unwrap().amount <= Uint128::zero()
    {
        return Err(StdError::generic_err("No sent value found."));
    }
    let sent_amount = sent_value.unwrap().amount;

    payments_store(&mut deps.storage, env.message.sender.clone(), sent_amount).unwrap();
    let adr = owner_cfg_read(&deps.storage)
        .load()
        .unwrap()
        .name_service_address;
    let msg = Register {
        name_c: Name {
            value: name,
            owner: deps.api.canonical_address(&env.message.sender)?,
        },
    };
    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: adr,
        msg: to_binary(&msg)?,
        send: vec![],
    });
    let res = HandleResponse {
        messages: vec![message],
        log: vec![],
        data: None,
    };

    Ok(res)
}
