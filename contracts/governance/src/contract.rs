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

pub fn handle_unsubscribe<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
) -> StdResult<HandleResponse> {
    let name_service_contract_address = owner_cfg_read(&deps.storage)
        .load()
        .unwrap()
        .name_service_address;
    let message = Deregister {
        name_c: Name {
            value: name,
            owner: deps.api.canonical_address(&env.message.sender).unwrap(),
        },
    };

    let exemessage = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: name_service_contract_address,
        msg: to_binary(&message).unwrap(),
        send: vec![],
    });
    let mut msgs: Vec<CosmosMsg> = vec![];
    msgs.push(exemessage);

    let paid_amount = payments_read(&deps.storage, env.message.sender.clone()).unwrap();
    if !paid_amount.is_zero() {
        let coin = Coin::new(paid_amount.u128() / 10, LUNA);

        msgs.push(
            BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: env.message.sender.clone(),
                amount: vec![coin],
            }
            .into(),
        );
        payments_delete(&mut deps.storage, env.message.sender).expect("No payments to delete");
    }
    let res = HandleResponse {
        messages: msgs,
        log: vec![],
        data: None,
    };
    Ok(res)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::PaidAmountIs { address } => to_binary(&try_paidamountis(deps, address)),
        QueryMsg::GetNameServiceAddress {} => to_binary(&get_nameservice_address(deps)),
        QueryMsg::AddressExists { address } => to_binary(&address_exists(deps, address)),
    }
}

pub fn address_exists<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<bool> {
    if payments_read(&deps.storage, address).is_err() {
        return Ok(false);
    }
    Ok(true)
}

pub fn try_paidamountis<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<Uint128> {
    let paid_amount = payments_read(&deps.storage, address)?;
    Ok(paid_amount)
}

pub fn get_nameservice_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<HumanAddr> {
    Ok(owner_cfg_read(&deps.storage)
        .load()
        .unwrap()
        .name_service_address)
}

#[cfg(tests)]
mod tests {
    use super::*;
    use crate::msg::HandleMsg::{Subscribe, Unsubscribe};
    use cosmwasm_std::coin;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let sent = Coin::new(1000, LUNA);
        let base_address = HumanAddr::from("test1");
        let env = mock_env(base_address.clone(), &[sent]);

        let msg1 = InitMsg {
            nameservice_code_id: 16,
        };
        let res1 = init(&mut deps, env.clone(), msg1);
        assert_eq!(&res1.is_err(), &false);
        let res1_message = res1.unwrap().messages.len();
        assert_eq!(res1_message, 1);

        let msg2 = Signup {};
        let _res2 = handle(&mut deps, env, msg2);
        let query = get_nameservice_address(&deps).unwrap();
        assert_eq!(query, base_address);
    }

    #[test]
    fn proper_subscription() {
        let mut deps = mock_dependencies(20, &[]);
        let sent = Coin::new(1000, LUNA);
        let base_address = HumanAddr::from("test1");
        let env = mock_env(base_address.clone(), &[sent]);

        let msg1 = InitMsg {
            nameservice_code_id: 16,
        };
        let _res1 = init(&mut deps, env.clone(), msg1);

        let msg2 = Signup {};
        let _res2 = handle(&mut deps, env.clone(), msg2);

        let msg3 = Subscribe {
            name: "Test1Name".to_string(),
        };
        let res3 = handle(&mut deps, env, msg3);
        assert_eq!(&res3.is_err(), &false);
        let res3_message = res3.unwrap().messages;
        assert_eq!(res3_message.len(), 1);

        if res3_message.len() == 1 {
            let intended_message: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: base_address.clone(),
                msg: to_binary(&Register {
                    name_c: Name {
                        value: "Test1Name".to_string(),
                        owner: deps.api.canonical_address(&base_address).unwrap(),
                    },
                })
                .unwrap(),
                send: vec![],
            })
            .into()];
            assert_eq!(intended_message, res3_message);
        }

        let query1 = address_exists(&deps, base_address.clone()).unwrap();
        assert_eq!(query1, true);

        let query2 = try_paidamountis(&deps, base_address).unwrap();
        assert_eq!(query2, Uint128(1000));
    }

    #[test]
    fn proper_unsubscription() {
        let mut deps = mock_dependencies(20, &[]);
        let sent = Coin::new(1000, LUNA);
        let base_address = HumanAddr::from("test1");
        let env = mock_env(base_address.clone(), &[sent]);

        let msg1 = InitMsg {
            nameservice_code_id: 16,
        };
        let _res1 = init(&mut deps, env.clone(), msg1);

        let msg2 = Signup {};
        let _res2 = handle(&mut deps, env.clone(), msg2);

        let msg3 = Subscribe {
            name: "Test1Name".to_string(),
        };
        let _res3 = handle(&mut deps, env.clone(), msg3);

        let msg4 = Unsubscribe {
            name: "Test1Name".to_string(),
        };
        let res4 = handle(&mut deps, env.clone(), msg4);
        assert_eq!(&res4.is_err(), &false);
        let res4_message = res4.unwrap().messages;
        assert_eq!(res4_message.len(), 2); //a bank message and a deregister one

        if res4_message.len() == 2 {
            let mut intended_messages: Vec<CosmosMsg> = vec![];

            let message1 = WasmMsg::Execute {
                contract_addr: base_address.clone(),
                msg: to_binary(&Deregister {
                    name_c: Name {
                        value: "Test1Name".to_string(),
                        owner: deps.api.canonical_address(&base_address).unwrap(),
                    },
                })
                .unwrap(),
                send: vec![],
            }
            .into();
            intended_messages.push(message1);
            let message2 = BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: base_address.clone(),
                amount: vec![coin(100, LUNA)],
            }
            .into();
            intended_messages.push(message2);

            assert_eq!(res4_message, intended_messages);
        }

        let query1 = address_exists(&deps, base_address).unwrap();
        assert_eq!(query1, false);
    }
}
