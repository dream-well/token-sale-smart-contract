use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, State};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage,
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
        viewing_key: msg.viewing_key.clone(),
    };

    config(&mut deps.storage).save(&state)?;

    // https://github.com/enigmampc/secret-toolkit/tree/master/packages/snip20
    // Register this contract to be able to receive the incentivized token
    // Enable this contract to see it's incentivized token details via viewing key
    let messages = vec![
        snip20::register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            1,
            msg.accepted_token.contract_hash.clone(),
            msg.accepted_token.address.clone(),
        )?,
        snip20::set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            RESPONSE_BLOCK_SIZE,
            msg.accepted_token.contract_hash,
            msg.accepted_token.address,
        )?,
        snip20::register_receive_msg(
            env.contract_code_hash,
            None,
            1,
            msg.offered_token.contract_hash.clone(),
            msg.offered_token.address.clone(),
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
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {}
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SecretContract;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::HumanAddr;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            accepted_token: SecretContract {
                address: HumanAddr("secretsefismartcontractaddress".to_string()),
                contract_hash: "sefismartcontracthash".to_string(),
            },
            offered_token: SecretContract {
                address: HumanAddr("secretbtntokensmartcontractaddress".to_string()),
                contract_hash: "btntokensmartcontracthash".to_string(),
            },
            viewing_key: "nannofromthegirlfromnowhereisathaidemon?".to_string(),
        };
        let env = mock_env("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        // Test that the 4 messages are created to receive and view for both tokens
        assert_eq!(4, res.messages.len());
    }
}
