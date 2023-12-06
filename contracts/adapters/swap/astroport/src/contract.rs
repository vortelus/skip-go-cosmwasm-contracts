use crate::{
    error::{ContractError, ContractResult},
    state::{ENTRY_POINT_CONTRACT_ADDRESS, ROUTER_CONTRACT_ADDRESS},
};
use astroport::{
    pair::{
        QueryMsg as PairQueryMsg, ReverseSimulationResponse, SimulationResponse,
        MAX_ALLOWED_SLIPPAGE,
    },
    router::ExecuteMsg as RouterExecuteMsg,
};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Api, Binary, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg};
use cw_utils::one_coin;
use skip::{
    asset::Asset,
    swap::{
        execute_transfer_funds_back, AstroportInstantiateMsg as InstantiateMsg, Cw20HookMsg,
        ExecuteMsg, QueryMsg, SimulateSwapExactAssetInResponse, SimulateSwapExactAssetOutResponse,
        SwapOperation,
    },
};

///////////////////
/// INSTANTIATE ///
///////////////////

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // Validate entry point contract address
    let checked_entry_point_contract_address =
        deps.api.addr_validate(&msg.entry_point_contract_address)?;

    // Store the entry point contract address
    ENTRY_POINT_CONTRACT_ADDRESS.save(deps.storage, &checked_entry_point_contract_address)?;

    // Validate router contract address
    let checked_router_contract_address = deps.api.addr_validate(&msg.router_contract_address)?;

    // Store the router contract address
    ROUTER_CONTRACT_ADDRESS.save(deps.storage, &checked_router_contract_address)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute(
            "entry_point_contract_address",
            checked_entry_point_contract_address.to_string(),
        )
        .add_attribute(
            "router_contract_address",
            checked_router_contract_address.to_string(),
        ))
}

///////////////
/// RECEIVE ///
///////////////

// Receive is the main entry point for the contract to
// receive cw20 tokens and execute the swap
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let sent_asset = Asset::Cw20(Cw20Coin {
        address: info.sender.to_string(),
        amount: cw20_msg.amount,
    });
    sent_asset.validate(&deps, &env, &info)?;

    // Set the sender to the originating address that triggered the cw20 send call
    // This is later validated / enforced to be the entry point contract address
    info.sender = deps.api.addr_validate(&cw20_msg.sender)?;

    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Swap { operations } => execute_swap(deps, env, info, sent_asset, operations),
    }
}

///////////////
/// EXECUTE ///
///////////////

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => receive_cw20(deps, env, info, cw20_msg),
        ExecuteMsg::Swap { operations } => {
            let sent_asset: Asset = one_coin(&info)?.into();
            execute_swap(deps, env, info, sent_asset, operations)
        }
        ExecuteMsg::TransferFundsBack {
            swapper,
            return_denom,
        } => Ok(execute_transfer_funds_back(
            deps,
            env,
            info,
            swapper,
            return_denom,
        )?),
    }
}

fn execute_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sent_asset: Asset,
    operations: Vec<SwapOperation>,
) -> ContractResult<Response> {
    // Get entry point contract address from storage
    let entry_point_contract_address = ENTRY_POINT_CONTRACT_ADDRESS.load(deps.storage)?;

    // Enforce the caller is the entry point contract
    if info.sender != entry_point_contract_address {
        return Err(ContractError::Unauthorized);
    }

    // Create the astroport swap message
    let swap_msg = create_astroport_swap_msg(
        deps.api,
        ROUTER_CONTRACT_ADDRESS.load(deps.storage)?,
        sent_asset,
        &operations,
    )?;

    let return_denom = match operations.last() {
        Some(last_op) => last_op.denom_out.clone(),
        None => return Err(ContractError::SwapOperationsEmpty),
    };

    // Create the transfer funds back message
    let transfer_funds_back_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::TransferFundsBack {
            swapper: entry_point_contract_address,
            return_denom,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(swap_msg)
        .add_message(transfer_funds_back_msg)
        .add_attribute("action", "dispatch_swap_and_transfer_back"))
}

////////////////////////
/// HELPER FUNCTIONS ///
////////////////////////

// Converts the swap operations to astroport AstroSwap operations
fn create_astroport_swap_msg(
    api: &dyn Api,
    router_contract_address: Addr,
    asset_in: Asset,
    swap_operations: &[SwapOperation],
) -> ContractResult<WasmMsg> {
    // Convert the swap operations to astroport swap operations
    let astroport_swap_operations = swap_operations
        .iter()
        .map(|swap_operation| swap_operation.into_astroport_swap_operation(api))
        .collect();

    // Create the astroport router execute message arguments
    let astroport_router_msg_args = RouterExecuteMsg::ExecuteSwapOperations {
        operations: astroport_swap_operations,
        minimum_receive: None,
        to: None,
        max_spread: Some(MAX_ALLOWED_SLIPPAGE.parse::<Decimal>()?),
    };

    // Create the astroport router swap message
    let swap_msg = asset_in.into_wasm_msg(
        router_contract_address.to_string(),
        to_binary(&astroport_router_msg_args)?,
    )?;

    Ok(swap_msg)
}

/////////////
/// QUERY ///
/////////////

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::RouterContractAddress {} => {
            to_binary(&ROUTER_CONTRACT_ADDRESS.load(deps.storage)?)
        }
        QueryMsg::SimulateSwapExactAssetIn {
            asset_in,
            swap_operations,
        } => to_binary(&query_simulate_swap_exact_asset_in(
            deps,
            asset_in,
            swap_operations,
        )?),
        QueryMsg::SimulateSwapExactAssetOut {
            asset_out,
            swap_operations,
        } => to_binary(&query_simulate_swap_exact_asset_out(
            deps,
            asset_out,
            swap_operations,
        )?),
        QueryMsg::SimulateSwapExactAssetInWithMetadata {
            asset_in,
            swap_operations,
            include_spot_price,
        } => to_binary(&query_simulate_swap_exact_asset_in_with_metadata(
            deps,
            asset_in,
            swap_operations,
            include_spot_price,
        )?),
        QueryMsg::SimulateSwapExactAssetOutWithMetadata {
            asset_out,
            swap_operations,
            include_spot_price,
        } => to_binary(&query_simulate_swap_exact_asset_out_with_metadata(
            deps,
            asset_out,
            swap_operations,
            include_spot_price,
        )?),
    }
    .map_err(From::from)
}

// Queries the astroport router contract to simulate a swap exact amount in
fn query_simulate_swap_exact_asset_in(
    deps: Deps,
    asset_in: Asset,
    swap_operations: Vec<SwapOperation>,
) -> ContractResult<Asset> {
    // Error if swap operations is empty
    let Some(first_op) = swap_operations.first() else {
        return Err(ContractError::SwapOperationsEmpty);
    };

    // Ensure asset_in's denom is the same as the first swap operation's denom in
    if asset_in.denom() != first_op.denom_in {
        return Err(ContractError::CoinInDenomMismatch);
    }

    let (asset_out, _) = simulate_swap_exact_asset_in(deps, asset_in, swap_operations, false)?;

    // Return the asset out
    Ok(asset_out)
}

// Queries the astroport pool contracts to simulate a multi-hop swap exact amount out
fn query_simulate_swap_exact_asset_out(
    deps: Deps,
    asset_out: Asset,
    swap_operations: Vec<SwapOperation>,
) -> ContractResult<Asset> {
    // Error if swap operations is empty
    let Some(last_op) = swap_operations.last() else {
        return Err(ContractError::SwapOperationsEmpty);
    };

    // Ensure asset_out's denom is the same as the last swap operation's denom out
    if asset_out.denom() != last_op.denom_out {
        return Err(ContractError::CoinOutDenomMismatch);
    }

    let (asset_in, _) = simulate_swap_exact_asset_out(deps, asset_out, swap_operations, false)?;

    // Return the asset in needed
    Ok(asset_in)
}

// Queries the astroport router contract to simulate a swap exact amount in with metadata
fn query_simulate_swap_exact_asset_in_with_metadata(
    deps: Deps,
    asset_in: Asset,
    swap_operations: Vec<SwapOperation>,
    include_spot_price: bool,
) -> ContractResult<SimulateSwapExactAssetInResponse> {
    // Error if swap operations is empty
    let Some(first_op) = swap_operations.first() else {
        return Err(ContractError::SwapOperationsEmpty);
    };

    // Ensure asset_in's denom is the same as the first swap operation's denom in
    if asset_in.denom() != first_op.denom_in {
        return Err(ContractError::CoinInDenomMismatch);
    }

    // Determine if we should request the simulation responses from simulate_swap_exact_asset_in
    let mut include_sim_resps = false;
    if include_spot_price {
        include_sim_resps = true;
    }

    // Simulate the swap exact amount in
    let (asset_out, sim_resps) = simulate_swap_exact_asset_in(
        deps,
        asset_in.clone(),
        swap_operations.clone(),
        include_sim_resps,
    )?;

    // Create the response
    let mut response = SimulateSwapExactAssetInResponse {
        asset_out,
        spot_price: None,
    };

    // Include the spot price in the response if requested
    if include_spot_price {
        response.spot_price = Some(calculate_spot_price_from_simulation_responses(
            deps,
            asset_in,
            swap_operations,
            sim_resps,
        )?)
    }

    Ok(response)
}

// Queries the astroport pool contracts to simulate a multi-hop swap exact amount out with metadata
fn query_simulate_swap_exact_asset_out_with_metadata(
    deps: Deps,
    asset_out: Asset,
    swap_operations: Vec<SwapOperation>,
    include_spot_price: bool,
) -> ContractResult<SimulateSwapExactAssetOutResponse> {
    // Error if swap operations is empty
    let Some(last_op) = swap_operations.last() else {
        return Err(ContractError::SwapOperationsEmpty);
    };

    // Ensure asset_out's denom is the same as the last swap operation's denom out
    if asset_out.denom() != last_op.denom_out {
        return Err(ContractError::CoinOutDenomMismatch);
    }

    // Determine if we should request the simulation responses from simulate_swap_exact_asset_out
    let mut include_sim_resps = false;
    if include_spot_price {
        include_sim_resps = true;
    }

    // Simulate the swap exact amount out
    let (asset_in, sim_resps) = simulate_swap_exact_asset_out(
        deps,
        asset_out.clone(),
        swap_operations.clone(),
        include_sim_resps,
    )?;

    // Create the response
    let mut response = SimulateSwapExactAssetOutResponse {
        asset_in,
        spot_price: None,
    };

    // Include the spot price in the response if requested
    if include_spot_price {
        response.spot_price = Some(calculate_spot_price_from_reverse_simulation_responses(
            deps,
            asset_out,
            swap_operations,
            sim_resps,
        )?)
    }

    Ok(response)
}

fn assert_max_spread(return_amount: Uint128, spread_amount: Uint128) -> ContractResult<()> {
    let max_spread = MAX_ALLOWED_SLIPPAGE.parse::<Decimal>()?;
    if Decimal::from_ratio(spread_amount, return_amount + spread_amount) > max_spread {
        return Err(ContractError::MaxSpreadAssertion {});
    }
    Ok(())
}

// Simulates a swap exact amount in request, returning the asset out and optionally the reverse simulation responses
fn simulate_swap_exact_asset_in(
    deps: Deps,
    asset_in: Asset,
    swap_operations: Vec<SwapOperation>,
    include_responses: bool,
) -> ContractResult<(Asset, Vec<SimulationResponse>)> {
    let (asset_out, responses) = swap_operations.iter().try_fold(
        (asset_in, Vec::new()),
        |(asset_out, mut responses), operation| -> Result<_, ContractError> {
            // Get the astroport offer asset type
            let astroport_offer_asset = asset_out.into_astroport_asset(deps.api)?;

            // Query the astroport pool contract to get the simulation response
            let res: SimulationResponse = deps.querier.query_wasm_smart(
                &operation.pool,
                &PairQueryMsg::Simulation {
                    offer_asset: astroport_offer_asset,
                    ask_asset_info: None,
                },
            )?;

            // Assert the operation does not exceed the max spread limit
            assert_max_spread(res.return_amount, res.spread_amount)?;

            if include_responses {
                responses.push(res.clone());
            }

            Ok((
                Asset::new(deps.api, &operation.denom_out, res.return_amount),
                responses,
            ))
        },
    )?;

    Ok((asset_out, responses))
}

// Simulates a swap exact amount out request, returning the asset in needed and optionally the reverse simulation responses
fn simulate_swap_exact_asset_out(
    deps: Deps,
    asset_out: Asset,
    swap_operations: Vec<SwapOperation>,
    include_responses: bool,
) -> ContractResult<(Asset, Vec<ReverseSimulationResponse>)> {
    let (asset_in, responses) = swap_operations.iter().rev().try_fold(
        (asset_out, Vec::new()),
        |(asset_in_needed, mut responses), operation| -> Result<_, ContractError> {
            // Get the astroport ask asset type
            let astroport_ask_asset = asset_in_needed.into_astroport_asset(deps.api)?;

            // Query the astroport pool contract to get the reverse simulation response
            let res: ReverseSimulationResponse = deps.querier.query_wasm_smart(
                &operation.pool,
                &PairQueryMsg::ReverseSimulation {
                    offer_asset_info: None,
                    ask_asset: astroport_ask_asset,
                },
            )?;

            // Assert the operation does not exceed the max spread limit
            assert_max_spread(res.offer_amount, res.spread_amount)?;

            if include_responses {
                responses.push(res.clone());
            }

            Ok((
                Asset::new(
                    deps.api,
                    &operation.denom_in,
                    res.offer_amount.checked_add(Uint128::one())?,
                ),
                responses,
            ))
        },
    )?;

    Ok((asset_in, responses))
}

// Calculate the spot price using simulation responses
fn calculate_spot_price_from_simulation_responses(
    deps: Deps,
    asset_in: Asset,
    swap_operations: Vec<SwapOperation>,
    simulation_responses: Vec<SimulationResponse>,
) -> ContractResult<Decimal> {
    let (_, spot_price) = swap_operations.iter().zip(simulation_responses).try_fold(
        (asset_in, Decimal::one()),
        |(asset_out, curr_spot_price), (op, res)| -> Result<_, ContractError> {
            // Calculate the amount out without slippage
            let amount_out_without_slippage = res
                .return_amount
                .checked_add(res.spread_amount)?
                .checked_add(res.commission_amount)?;

            Ok((
                Asset::new(deps.api, &op.denom_out, res.return_amount),
                curr_spot_price.checked_mul(Decimal::from_ratio(
                    amount_out_without_slippage,
                    asset_out.amount(),
                ))?,
            ))
        },
    )?;

    Ok(spot_price)
}

// Calculates the spot price using reverse simulaation responses
fn calculate_spot_price_from_reverse_simulation_responses(
    deps: Deps,
    asset_out: Asset,
    swap_operations: Vec<SwapOperation>,
    reverse_simulation_responses: Vec<ReverseSimulationResponse>,
) -> ContractResult<Decimal> {
    let (_, spot_price) = swap_operations
        .iter()
        .rev()
        .zip(reverse_simulation_responses)
        .try_fold(
            (asset_out, Decimal::one()),
            |(asset_in_needed, curr_spot_price), (op, res)| -> Result<_, ContractError> {
                let amount_out_without_slippage = asset_in_needed
                    .amount()
                    .checked_add(res.spread_amount)?
                    .checked_add(res.commission_amount)?;

                Ok((
                    Asset::new(
                        deps.api,
                        &op.denom_in,
                        res.offer_amount.checked_add(Uint128::one())?,
                    ),
                    curr_spot_price.checked_mul(Decimal::from_ratio(
                        amount_out_without_slippage,
                        res.offer_amount,
                    ))?,
                ))
            },
        )?;

    Ok(spot_price)
}
