use crate::msg::{HandleMsg, NSInitMsg, QueryMsg};
use crate::state::{names_read, names_store, nsowner_read, nsowner_store, NSOwner, Name, NamesS};
use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, WasmMsg,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: NSInitMsg,
) -> StdResult<InitResponse> {
    let names_storage = NamesS {
        names_vector: vec![],
    };
    names_store(&mut deps.storage).save(&names_storage).unwrap();
    let nsowner = NSOwner {
        nameservice_owner: env.message.sender,
    };
    nsowner_store(&mut deps.storage).save(&nsowner).unwrap();
    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.hook.clone().unwrap().contract_addr,
        msg: msg.hook.unwrap().msg,
        send: vec![],
    });
    let response = InitResponse {
        messages: vec![message],
        log: vec![],
    };
    Ok(response)
}