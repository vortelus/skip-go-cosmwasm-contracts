use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Addr, Coin,
    ReplyOn::Never,
    SubMsg, WasmMsg,
};
use osmosis_std::types::cosmos::base::v1beta1::Coin as OsmosisStdCoin;
use osmosis_std::types::osmosis::poolmanager::v1beta1::{MsgSwapExactAmountIn, SwapAmountInRoute};
use skip::swap::{ExecuteMsg, SwapOperation};
use skip_swap_osmosis_poolmanager_swap::error::ContractResult;
use test_case::test_case;

/*
Test Cases:

Expect Success
    - One Swap Operation
    - Multiple Swap Operations
    - No Swap Operations (This is prevented in the entry point contract; and will fail on Osmosis module if attempted)

Expect Error
    - No Coin Sent
    - More Than One Coin Sent
    - Invalid Pool ID Conversion For Swap Operations

 */

// Define test parameters
struct Params {
    info_funds: Vec<Coin>,
    swap_operations: Vec<SwapOperation>,
    expected_messages: Vec<SubMsg>,
    expected_error_string: String,
}

// Test execute_swap
#[test_case(
    Params {
        info_funds: vec![Coin::new(100, "uosmo")],
        swap_operations: vec![
            SwapOperation {
                pool: "1".to_string(),
                denom_in: "uosmo".to_string(),
                denom_out: "uatom".to_string(),
            }
        ],
        expected_messages: vec![
            SubMsg {
                id: 0,
                msg: MsgSwapExactAmountIn {
                    sender: "swap_contract_address".to_string(),
                    routes: vec![
                        SwapAmountInRoute {
                            pool_id: 1,
                            token_out_denom: "uatom".to_string(),
                        }
                    ],
                    token_in: Some(
                        OsmosisStdCoin {
                            denom: "uosmo".to_string(),
                            amount: "100".to_string(),
                        }
                    ),
                    token_out_min_amount: "1".to_string(),
                }
                .into(),
                gas_limit: None,
                reply_on: Never,
            },
            SubMsg {
                id: 0,
                msg: WasmMsg::Execute {
                    contract_addr: "swap_contract_address".to_string(),
                    msg: to_binary(&ExecuteMsg::TransferFundsBack {
                        swapper: Addr::unchecked("swapper"),
                    })?,
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
                reply_on: Never,
            },
        ],
        expected_error_string: "".to_string(),
    };
"One Swap Operation")]
#[test_case(
    Params {
        info_funds: vec![Coin::new(100, "uosmo")],
        swap_operations: vec![
            SwapOperation {
                pool: "1".to_string(),
                denom_in: "uosmo".to_string(),
                denom_out: "uatom".to_string(),
            },
            SwapOperation {
                pool: "2".to_string(),
                denom_in: "uatom".to_string(),
                denom_out: "untrn".to_string(),
            }
        ],
        expected_messages: vec![
            SubMsg {
                id: 0,
                msg: MsgSwapExactAmountIn {
                    sender: "swap_contract_address".to_string(),
                    routes: vec![
                        SwapAmountInRoute {
                            pool_id: 1,
                            token_out_denom: "uatom".to_string(),
                        },
                        SwapAmountInRoute {
                            pool_id: 2,
                            token_out_denom: "untrn".to_string(),
                        }
                    ],
                    token_in: Some(
                        OsmosisStdCoin {
                            denom: "uosmo".to_string(),
                            amount: "100".to_string(),
                        }
                    ),
                    token_out_min_amount: "1".to_string(),
                }
                .into(),
                gas_limit: None,
                reply_on: Never,
            },
            SubMsg {
                id: 0,
                msg: WasmMsg::Execute {
                    contract_addr: "swap_contract_address".to_string(),
                    msg: to_binary(&ExecuteMsg::TransferFundsBack {
                        swapper: Addr::unchecked("swapper"),
                    })?,
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
                reply_on: Never,
            },
        ],
        expected_error_string: "".to_string(),
    };
"Multiple Swap Operations")]
#[test_case(
    Params {
        info_funds: vec![Coin::new(100, "uosmo")],
        swap_operations: vec![],
        expected_messages: vec![
            SubMsg {
                id: 0,
                msg: MsgSwapExactAmountIn {
                    sender: "swap_contract_address".to_string(),
                    routes: vec![],
                    token_in: Some(
                        OsmosisStdCoin {
                            denom: "uosmo".to_string(),
                            amount: "100".to_string(),
                        }
                    ),
                    token_out_min_amount: "1".to_string(),
                }
                .into(),
                gas_limit: None,
                reply_on: Never,
            },
            SubMsg {
                id: 0,
                msg: WasmMsg::Execute {
                    contract_addr: "swap_contract_address".to_string(),
                    msg: to_binary(&ExecuteMsg::TransferFundsBack {
                        swapper: Addr::unchecked("swapper"),
                    })?,
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
                reply_on: Never,
            },
        ],
        expected_error_string: "".to_string(),
    };
"No Swap Operations")]
#[test_case(
    Params {
        info_funds: vec![],
        swap_operations: vec![
            SwapOperation {
                pool: "pool_1".to_string(),
                denom_in: "uosmo".to_string(),
                denom_out: "uatom".to_string(),
            }
        ],
        expected_messages: vec![],
        expected_error_string: "No funds sent".to_string(),
    };
    "No Coin Sent - Expect Error")]
#[test_case(
    Params {
        info_funds: vec![
            Coin::new(100, "uosmo"),
            Coin::new(100, "uatom"),
        ],
        swap_operations: vec![
            SwapOperation {
                pool: "pool_1".to_string(),
                denom_in: "uosmo".to_string(),
                denom_out: "uatom".to_string(),
            }
        ],
        expected_messages: vec![],
        expected_error_string: "Sent more than one denomination".to_string(),
    };
    "More Than One Coin Sent - Expect Error")]
#[test_case(
    Params {
        info_funds: vec![Coin::new(100, "uosmo")],
        swap_operations: vec![
            SwapOperation {
                pool: "pool_1".to_string(),
                denom_in: "uosmo".to_string(),
                denom_out: "uatom".to_string(),
            }
        ],
        expected_messages: vec![],
        expected_error_string: "Parse Int error raised: invalid pool String to pool id u64 conversion".to_string(),
    };
    "Invalid Pool ID Conversion For Swap Operations - Expect Error")]
fn test_execute_swap(params: Params) -> ContractResult<()> {
    // Create mock dependencies
    let mut deps = mock_dependencies();

    // Create mock env
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("swap_contract_address");

    // Convert info funds vector into a slice of Coin objects
    let info_funds: &[Coin] = &params.info_funds;

    // Create mock info with entry point contract address
    let info = mock_info("swapper", info_funds);

    // Call execute_swap with the given test parameters
    let res = skip_swap_osmosis_poolmanager_swap::contract::execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::Swap {
            operations: params.swap_operations.clone(),
        },
    );

    // Assert the behavior is correct
    match res {
        Ok(res) => {
            // Assert the test did not expect an error
            assert!(
                params.expected_error_string.is_empty(),
                "expected test to error with {:?}, but it succeeded",
                params.expected_error_string
            );

            // Assert the messages are correct
            assert_eq!(res.messages, params.expected_messages);
        }
        Err(err) => {
            // Assert the test expected an error
            assert!(
                !params.expected_error_string.is_empty(),
                "expected test to succeed, but it errored with {:?}",
                err
            );

            // Assert the error is correct
            assert_eq!(err.to_string(), params.expected_error_string);
        }
    }

    Ok(())
}
