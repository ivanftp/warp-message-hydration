#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::error::ContractError;
use crate::msg::{InputVariable, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State { owner: msg.owner };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetHydratedMsg {
            msg_template,
            msg_params,
        } => to_binary(&query::hydrate(msg_template, msg_params)?),
    }
}

pub mod query {
    use super::*;
    use crate::helpers::get_hydrated_string;

    pub fn hydrate(msg_template: String, msg_params: Vec<InputVariable>) -> StdResult<CosmosMsg> {
        // Call get_hydrated_string and manually handle the Result
        let hydrated_str = match get_hydrated_string(msg_template, msg_params) {
            Ok(s) => s,
            // Convert the error to a StdError before returning it
            Err(e) => {
                return Err(StdError::generic_err(format!(
                    "Failed to hydrate string: {}",
                    e
                )))
            }
        };

        let hydrated_msg_result: Result<CosmosMsg, _> = serde_json_wasm::from_str(&hydrated_str);

        match hydrated_msg_result {
            Ok(msg) => Ok(msg),
            Err(_) => Err(StdError::generic_err(
                "Error hydrating message. Please check message template and parameters.",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::decode;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, WasmMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Addr::unchecked("owner"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn hydrate_nested() {
        // This is a test for the hydration function when there are nested base64 messages.

        // Create a mocked dependencies object for the test.
        let mut deps = mock_dependencies();

        // Initialize the test scenario by instantiating the contract
        // with a mocked environment and funds.
        let msg = InstantiateMsg {
            owner: Addr::unchecked("owner"),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Declare input variables for hydration.
        // These will replace placeholders in the message template.
        let msg_params = vec![
            InputVariable {
                key: "$warp.var.variable1".to_string(),
                value: "terra12345".to_string(),
            },
            InputVariable {
                key: "$warp.var.variable2".to_string(),
                value: "uterra".to_string(),
            },
            InputVariable {
                key: "$warp.var.variable3".to_string(),
                value: "54321".to_string(),
            },
            InputVariable {
                key: "$warp.var.variable4".to_string(),
                value: "terra11111".to_string(),
            },
            InputVariable {
                key: "$warp.var.variable5".to_string(),
                value: "0.05".to_string(),
            },
        ];

        // Define a message template that contains placeholders.
        // The placeholders are in the form of $warp.var.variableX,
        // where X is the index of the variable.
        let msg_template = r#"{
            "wasm": {
              "execute": {
                "contract_addr": "$warp.var.variable1",
                "msg": "eyJzZW5kIjp7ImNvbnRyYWN0IjoidGVycmE1NDMyMSIsImFtb3VudCI6IjEyMzQ1IiwibXNnIjoiZXlKbGVHVmpkWFJsWDNOM1lYQmZiM0JsY21GMGFXOXVjeUk2ZXlKdmNHVnlZWFJwYjI1eklqcGJleUpoYzNSeWIxOXpkMkZ3SWpwN0ltOW1abVZ5WDJGemMyVjBYMmx1Wm04aU9uc2lkRzlyWlc0aU9uc2lZMjl1ZEhKaFkzUmZZV1JrY2lJNklpUjNZWEp3TG5aaGNpNTJZWEpwWVdKc1pURWlmWDBzSW1GemExOWhjM05sZEY5cGJtWnZJanA3SW01aGRHbDJaVjkwYjJ0bGJpSTZleUprWlc1dmJTSTZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVElpZlgxOWZWMHNJbTFwYm1sdGRXMWZjbVZqWldsMlpTSTZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVE1pTENKMGJ5STZJaVIzWVhKd0xuWmhjaTUyWVhKcFlXSnNaVFFpTENKdFlYaGZjM0J5WldGa0lqb2lKSGRoY25BdWRtRnlMblpoY21saFlteGxOU0o5ZlE9PSJ9fQ==",
                "funds": []
              }
            }
          }
          "#.to_string();

        // Call the function under test.
        // In this case, the hydrate function is called
        // by executing a query to get the hydrated message.
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetHydratedMsg {
                msg_template,
                msg_params,
            },
        )
        .unwrap();

        // Convert the result back into a CosmosMsg, which is the expected type.
        let hydrated_msg: CosmosMsg = from_binary(&res).unwrap();

        // Define the expected result of the test.
        // This is the message that we expect to get after hydration.
        let expected_result: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "terra12345".to_string(),
            msg: cosmwasm_std::Binary(decode("eyJzZW5kIjp7ImNvbnRyYWN0IjoidGVycmE1NDMyMSIsImFtb3VudCI6IjEyMzQ1IiwibXNnIjoiZXlKbGVHVmpkWFJsWDNOM1lYQmZiM0JsY21GMGFXOXVjeUk2ZXlKdmNHVnlZWFJwYjI1eklqcGJleUpoYzNSeWIxOXpkMkZ3SWpwN0ltOW1abVZ5WDJGemMyVjBYMmx1Wm04aU9uc2lkRzlyWlc0aU9uc2lZMjl1ZEhKaFkzUmZZV1JrY2lJNkluUmxjbkpoTVRJek5EVWlmWDBzSW1GemExOWhjM05sZEY5cGJtWnZJanA3SW01aGRHbDJaVjkwYjJ0bGJpSTZleUprWlc1dmJTSTZJblYwWlhKeVlTSjlmWDE5WFN3aWJXbHVhVzExYlY5eVpXTmxhWFpsSWpvaU5UUXpNakVpTENKMGJ5STZJblJsY25KaE1URXhNVEVpTENKdFlYaGZjM0J5WldGa0lqb2lNQzR3TlNKOWZRPT0ifX0=").unwrap()),
            funds: vec![]
        });

        // Finally, compare the actual hydrated message to the expected result.
        // The test passes if they are equal, and fails otherwise.
        assert_eq!(hydrated_msg, expected_result);
    }

    #[test]
    fn hydrate_basic_no_nested() {
        // This is a test for the hydration function for a basic case when there are no nested base64 messages.

        // Create a mock environment
        let mut deps = mock_dependencies();

        // Initialize the smart contract
        let msg = InstantiateMsg {
            owner: Addr::unchecked("owner"),
        };
        let info = mock_info("creator", &coins(2, "token"));

        // Instantiate the smart contract
        // unwrap() is used to assert the operation doesn't return an error.
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Define input parameters for hydration
        let msg_params = vec![
            InputVariable {
                key: "$warp.var.variable1".to_string(),
                value: "terra987".to_string(),
            },
            InputVariable {
                key: "$warp.var.variable2".to_string(),
                value: "[]".to_string(),
            },
        ];

        // Template message
        // base64 msg is {"transfer":{"recipient":"creator","amount":"12345"}}}
        let msg_template = r#"{
            "wasm": {
              "execute": {
                "contract_addr": "$warp.var.variable1",
                "msg": "eyJ0cmFuc2ZlciI6eyJyZWNpcGllbnQiOiJjcmVhdG9yIiwiYW1vdW50IjoiMTIzNDUifX19",
                "funds": $warp.var.variable2
              }
            }
          }
          "#
        .to_string();

        // Query to hydrate the message using the mock environment
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetHydratedMsg {
                msg_template,
                msg_params,
            },
        )
        .unwrap();

        // Convert the response from binary to the CosmosMsg type
        let hydrated_msg: CosmosMsg = from_binary(&res).unwrap();

        // Expected result after hydration
        let expected_result: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "terra987".to_string(),
            msg: cosmwasm_std::Binary(
                decode("eyJ0cmFuc2ZlciI6eyJyZWNpcGllbnQiOiJjcmVhdG9yIiwiYW1vdW50IjoiMTIzNDUifX19")
                    .unwrap(),
            ),
            funds: vec![],
        });

        // Assert equality of the hydrated message and expected result
        assert_eq!(hydrated_msg, expected_result);
    }

    #[test]
    fn hydrate_empty_params() {
        // This is a test for the hydration function when the input parameters are empty.

        // Create a mock environment
        let mut deps = mock_dependencies();

        // Initialize the smart contract
        let msg = InstantiateMsg {
            owner: Addr::unchecked("owner"),
        };
        let info = mock_info("creator", &coins(2, "token"));

        // Instantiate the smart contract
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Define input parameters for hydration
        let msg_params = vec![];

        // Template message
        // base64 msg is {"transfer":{"recipient":"creator","amount":"12345"}}}
        let msg_template = r#"{
            "wasm": {
              "execute": {
                "contract_addr": "terra987",
                "msg": "eyJ0cmFuc2ZlciI6eyJyZWNpcGllbnQiOiJjcmVhdG9yIiwiYW1vdW50IjoiMTIzNDUifX19",
                "funds": []
              }
            }
          }
          "#
        .to_string();

        // Query to hydrate the message using the mock environment
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetHydratedMsg {
                msg_template,
                msg_params,
            },
        )
        .unwrap();

        // Convert the response from binary to the CosmosMsg type
        let hydrated_msg: CosmosMsg = from_binary(&res).unwrap();

        // Expected result after hydration
        let expected_result: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "terra987".to_string(),
            msg: cosmwasm_std::Binary(
                decode("eyJ0cmFuc2ZlciI6eyJyZWNpcGllbnQiOiJjcmVhdG9yIiwiYW1vdW50IjoiMTIzNDUifX19")
                    .unwrap(),
            ),
            funds: vec![],
        });

        // Assert equality of the hydrated message and expected result
        assert_eq!(hydrated_msg, expected_result);
    }

    #[test]
    fn hydrate_partially_used_params() {
        // This is a test for the hydration function when there are partially used parameters and some unused parameters.

        // Create a mock environment
        let mut deps = mock_dependencies();

        // Initialize the smart contract
        let msg = InstantiateMsg {
            owner: Addr::unchecked("owner"),
        };
        let info = mock_info("creator", &coins(2, "token"));

        // Instantiate the smart contract
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Define input parameters for hydration
        let msg_params = vec![
            InputVariable {
                key: "$warp.var.variable1".to_string(),
                value: "terra987".to_string(),
            },
            InputVariable {
                key: "$warp.var.variable2".to_string(),
                value: "[]".to_string(),
            },
        ];

        // Template message
        // base64 msg is {"transfer":{"recipient":"creator","amount":"12345"}}}
        let msg_template = r#"{
            "wasm": {
              "execute": {
                "contract_addr": "$warp.var.variable1",
                "msg": "eyJ0cmFuc2ZlciI6eyJyZWNpcGllbnQiOiJjcmVhdG9yIiwiYW1vdW50IjoiMTIzNDUifX19",
                "funds": []
              }
            }
          }
          "#
        .to_string();

        // Query to hydrate the message using the mock environment
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetHydratedMsg {
                msg_template,
                msg_params,
            },
        )
        .unwrap();

        // Convert the response from binary to the CosmosMsg type
        let hydrated_msg: CosmosMsg = from_binary(&res).unwrap();

        // Expected result after hydration
        let expected_result: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "terra987".to_string(),
            msg: cosmwasm_std::Binary(
                decode("eyJ0cmFuc2ZlciI6eyJyZWNpcGllbnQiOiJjcmVhdG9yIiwiYW1vdW50IjoiMTIzNDUifX19")
                    .unwrap(),
            ),
            funds: vec![],
        });

        // Assert equality of the hydrated message and expected result
        assert_eq!(hydrated_msg, expected_result);
    }
}
