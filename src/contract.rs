use crate::msg::{BalanceResponse, ConfigResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::snip20;

pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        accepted_token: msg.accepted_token.clone(),
        offered_token: msg.offered_token.clone(),
        admin: env.message.sender.clone(),
        exchange_rate: msg.exchange_rate,
        contract_address: env.contract.address,
        total_raised: Uint128(0),
        viewing_key: msg.viewing_key.clone(),
    };

    config(&mut deps.storage).save(&state)?;

    // https://github.com/enigmampc/secret-toolkit/tree/master/packages/snip20
    // Register this contract to be able to receive the ACCEPTED token
    // Enable this contract to see it's OFFERED token
    let messages = vec![
        snip20::register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            1,
            msg.accepted_token.contract_hash.clone(),
            msg.accepted_token.address.clone(),
        )?,
        snip20::set_viewing_key_msg(
            msg.viewing_key,
            None,
            RESPONSE_BLOCK_SIZE,
            msg.offered_token.contract_hash,
            msg.offered_token.address,
        )?,
    ];

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receive(deps, env, from, amount, msg),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&public_config(deps)?),
        QueryMsg::OfferedTokenAvailable {} => to_binary(&offered_token_available(deps)?),
    }
}

fn public_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(ConfigResponse {
        accepted_token: state.accepted_token,
        offered_token: state.offered_token,
        admin: state.admin,
        exchange_rate: state.exchange_rate,
        contract_address: state.contract_address,
        total_raised: state.total_raised,
    })
}

fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    _msg: Binary,
) -> StdResult<HandleResponse> {
    // Ensure that the sent tokens are from an expected contract address
    let mut state = config_read(&deps.storage).load()?;
    if env.message.sender != state.accepted_token.address {
        return Err(StdError::generic_err(format!(
            "This token is not supported. Supported: {}, given: {}",
            state.accepted_token.address, env.message.sender
        )));
    }

    state.total_raised = state.total_raised + amount;
    config(&mut deps.storage).save(&state)?;

    // apply exchange rate to amount
    let amount_of_offered_token_to_send = Uint128(amount.u128() * state.exchange_rate.u128());

    // Transfer offered token to user
    let messages = vec![
        snip20::transfer_msg(
            from,
            amount_of_offered_token_to_send,
            None,
            RESPONSE_BLOCK_SIZE,
            state.offered_token.contract_hash,
            state.offered_token.address,
        )?,
        snip20::transfer_msg(
            state.admin,
            amount,
            None,
            RESPONSE_BLOCK_SIZE,
            state.accepted_token.contract_hash,
            state.accepted_token.address,
        )?,
    ];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}

fn offered_token_available<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<BalanceResponse> {
    let state = config_read(&deps.storage).load()?;
    let balance = snip20::balance_query(
        &deps.querier,
        state.contract_address,
        state.viewing_key,
        RESPONSE_BLOCK_SIZE,
        state.offered_token.contract_hash,
        state.offered_token.address,
    )?;
    Ok(BalanceResponse {
        amount: balance.amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ReceiveMsg;
    use crate::state::SecretContract;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};

    pub const MOCK_ADMIN: &str = "admin";
    pub const MOCK_ACCEPTED_TOKEN_ADDRESS: &str = "sefismartcontractaddress";
    pub const MOCK_ACCEPTED_TOKEN_CONTRACT_HASH: &str = "sefismartcontracthash";
    pub const MOCK_OFFERED_TOKEN_ADDRESS: &str = "btnsmartcontractaddress";
    pub const MOCK_OFFERED_TOKEN_CONTRACT_HASH: &str = "btnsmartcontracthash";

    // === HELPERS ===
    fn init_helper() -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let env = mock_env(MOCK_ADMIN, &[]);
        let accepted_token = SecretContract {
            address: HumanAddr::from(MOCK_ACCEPTED_TOKEN_ADDRESS),
            contract_hash: MOCK_ACCEPTED_TOKEN_CONTRACT_HASH.to_string(),
        };
        let offered_token = SecretContract {
            address: HumanAddr::from(MOCK_OFFERED_TOKEN_ADDRESS),
            contract_hash: MOCK_OFFERED_TOKEN_CONTRACT_HASH.to_string(),
        };
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            accepted_token: accepted_token.clone(),
            offered_token: offered_token.clone(),
            exchange_rate: Uint128(123),
            viewing_key: "nannofromthegirlfromnowhereisathaidemon?".to_string(),
        };
        (init(&mut deps, env.clone(), msg), deps)
    }

    #[test]
    fn test_public_config() {
        let (_init_result, deps) = init_helper();

        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        // Test response does not include viewing key.
        // Test that the desired fields are returned.
        let accepted_token = SecretContract {
            address: HumanAddr::from(MOCK_ACCEPTED_TOKEN_ADDRESS),
            contract_hash: MOCK_ACCEPTED_TOKEN_CONTRACT_HASH.to_string(),
        };
        let offered_token = SecretContract {
            address: HumanAddr::from(MOCK_OFFERED_TOKEN_ADDRESS),
            contract_hash: MOCK_OFFERED_TOKEN_CONTRACT_HASH.to_string(),
        };
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(
            ConfigResponse {
                accepted_token: accepted_token,
                offered_token: offered_token,
                admin: HumanAddr::from(MOCK_ADMIN),
                contract_address: state.contract_address,
                exchange_rate: Uint128(123),
                total_raised: Uint128(0)
            },
            value
        );
    }

    #[test]
    fn test_receive_accepted_token_callback() {
        let (_init_result, mut deps) = init_helper();
        let amount: Uint128 = Uint128(333);
        let from: HumanAddr = HumanAddr::from("someuser");

        // Test that only accepted token is accepted
        let msg = HandleMsg::Receive {
            amount: amount,
            from: from.clone(),
            sender: from,
            msg: to_binary(&ReceiveMsg::Deposit {}).unwrap(),
        };
        let handle_response = handle(
            &mut deps,
            mock_env(MOCK_OFFERED_TOKEN_ADDRESS, &[]),
            msg.clone(),
        );
        assert_eq!(
            handle_response.unwrap_err(),
            StdError::GenericErr {
                msg: format!(
                    "This token is not supported. Supported: {}, given: {}",
                    MOCK_ACCEPTED_TOKEN_ADDRESS, MOCK_OFFERED_TOKEN_ADDRESS
                ),
                backtrace: None
            }
        );

        // Test that a request is sent to the offered token contract address to transfer tokens to the sender
        let handle_response = handle(
            &mut deps,
            mock_env(MOCK_ACCEPTED_TOKEN_ADDRESS, &[]),
            msg.clone(),
        );
        let res = handle_response.unwrap();
        assert_eq!(2, res.messages.len());

        // Test that total raised is updated
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(amount, value.total_raised);
    }
}
