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

pub fn try_valueis<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<String> {
    let mut def_v = String::default();
    let names_iter = names_read(&deps.storage).load()?.names_vector;
    for val in names_iter {
        if val.owner == deps.api.canonical_address(&owner)? {
            def_v = val.value;
            break;
        }
    }
    Ok(def_v)
}

#[cfg(tests)]
mod tests {
    use super::*;
    use crate::msg::HandleMsg::{Deregister, Register, TestPurposes};
    use crate::msg::InitHook;
    use crate::msg::QueryMsg::{NameExists, OwnerIs, ValueIs};
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn proper_initialization() {
        let operator_address = HumanAddr::from("test1");
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(operator_address.clone(), &[]);

        let msg1 = NSInitMsg {
            hook: Some(InitHook {
                contract_addr: env.clone().contract.address,
                msg: to_binary(&TestPurposes {}).unwrap(),
            }),
        };
        let res1 = init(&mut deps, env.clone(), msg1);
        assert_eq!(&res1.is_err(), &false);
        let res1_message = res1.unwrap().messages.len();
        assert_eq!(res1_message, 1);
    }

    #[test]
    fn proper_registration() {
        let operator_address = HumanAddr::from("test1");
        let test_name: String = "Test1Name".to_string();
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(operator_address.clone(), &[]);
        let msg1 = NSInitMsg {
            hook: Some(InitHook {
                contract_addr: env.clone().contract.address,
                msg: to_binary(&TestPurposes {}).unwrap(),
            }),
        };
        let _res1 = init(&mut deps, env.clone(), msg1);

        let msg2 = Register {
            name_c: Name {
                value: test_name.clone(),
                owner: deps
                    .api
                    .canonical_address(&operator_address.clone())
                    .unwrap(),
            },
        };
        let res2 = handle(&mut deps, env.clone(), msg2);
        assert_eq!(&res2.is_err(), &false);
        let res2_message = res2.unwrap().messages.len();
        assert_eq!(res2_message, 0);

        let msg3 = NameExists {
            value: test_name.clone(),
        };
        let res3 = query(&deps, msg3).unwrap();
        let res3_value: StdResult<bool> = from_binary(&res3).unwrap();
        assert_eq!(true, res3_value.unwrap());

        let msg4 = OwnerIs {
            value: test_name.clone(),
        };
        let res4 = query(&deps, msg4).unwrap();
        let res4_value: StdResult<HumanAddr> = from_binary(&res4);
        assert_eq!(res4_value.unwrap(), operator_address.clone());

        let msg5 = ValueIs {
            owner: operator_address,
        };
        let res5 = query(&deps, msg5).unwrap();
        let res5_value: StdResult<String> = from_binary(&res5);
        assert_eq!(res5_value.unwrap(), test_name);
    }

}