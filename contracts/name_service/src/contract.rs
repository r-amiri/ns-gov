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

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Register {
            name_c: name_component,
        } => try_register(deps, env, name_component),
        HandleMsg::Deregister {
            name_c: name_component,
        } => try_deregister(deps, env, name_component),
        HandleMsg::TestPurposes {} => test_purposes(),
    }
}

pub fn test_purposes() -> StdResult<HandleResponse> {
    Ok(Default::default())
}

pub fn try_register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name_c: Name,
) -> StdResult<HandleResponse> {
    if env.message.sender
        != nsowner_read(&deps.storage)
        .load()
        .unwrap()
        .nameservice_owner
    {
        return Err(StdError::generic_err("Access not granted."));
    }
    let mut found = false;
    let names_iter = names_read(&deps.storage).load()?.names_vector;
    for val in names_iter {
        if val.value == name_c.value {
            found = true;
            break;
        }
    }
    if !found {
        names_store(&mut deps.storage).update(|mut names_s| {
            names_s.names_vector.push(name_c);
            Ok(names_s)
        })?;
    }
    Ok(HandleResponse::default())
}

pub fn try_deregister<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name_component: Name,
) -> StdResult<HandleResponse> {
    if env.message.sender
        != nsowner_read(&deps.storage)
        .load()
        .unwrap()
        .nameservice_owner
    {
        return Err(StdError::generic_err("Access not granted"));
    }
    let names_iter = names_read(&deps.storage).load()?.names_vector;
    for (index, val) in names_iter.into_iter().enumerate() {
        if val == name_component {
            names_store(&mut deps.storage).update(|mut names_s| {
                names_s.names_vector.remove(index);
                Ok(names_s)
            })?;
        }
    }
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::NameExists { value } => to_binary(&try_nameexists(deps, value)),
        QueryMsg::OwnerIs { value } => to_binary(&try_owneris(deps, value)?),
        QueryMsg::ValueIs { owner } => to_binary(&try_valueis(deps, owner)?),
    }
}

pub fn try_nameexists<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    value: String,
) -> StdResult<bool> {
    let mut found = false;
    let names_iter = names_read(&deps.storage).load().unwrap().names_vector;
    for val in names_iter {
        if val.value == value {
            found = true;
            break;
        }
    }
    Ok(found)
}

pub fn try_owneris<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    value: String,
) -> StdResult<HumanAddr> {
    let mut def_h_a = HumanAddr::default();
    let names_iter = names_read(&deps.storage).load()?.names_vector;
    for val in names_iter {
        if val.value == value {
            def_h_a = deps.api.human_address(&val.owner).unwrap();
            break;
        }
    }
    Ok(def_h_a)
}
