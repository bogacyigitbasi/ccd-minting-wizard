use mint_wizard::{ContractBalanceOfQueryParams, ContractBalanceOfQueryResponse, *};
use concordium_cis2::*;
use concordium_smart_contract_testing::*;
use concordium_std::{
    collections::BTreeMap, AccountSignatures, CredentialSignatures, HashSha2256,
    SignatureEd25519, Timestamp,
};
use concordium_std_derive::*;

/// The tests accounts.
const ALICE: AccountAddress =
    account_address!("2wkBET2rRgE8pahuaczxKbmv7ciehqsne57F9gtzf1PVdr2VP3");
const ALICE_ADDR: Address = Address::Account(ALICE);
const BOB: AccountAddress = account_address!("2xBpaHottqhwFZURMZW4uZduQvpxNDSy46iXMYs9kceNGaPpZX");
const BOB_ADDR: Address = Address::Account(BOB);

const UPGRADER: AccountAddress =
    account_address!("2xdTv8awN1BjgYEw8W1BVXVtiEwG2b29U8KoZQqJrDuEqddseE");
const UPGRADER_ADDR: Address = Address::Account(UPGRADER);
const PAUSER: AccountAddress =
    account_address!("2yWkbp92JL9LYVmxgP1QfTDsJs9sMLAWJBYMy8md3SQz5ErzEd");
const PAUSER_ADDR: Address = Address::Account(PAUSER);


/// Token IDs.
const TOKEN_0: ContractTokenId = TokenIdU8(2);
const TOKEN_1: ContractTokenId = TokenIdU8(42);

const TOKEN_0_METADATA: &str = "https://some.example/token/2A";
const TOKEN_1_METADATA: &str = "https://some.example/token/3F";

/// Initial balance of the accounts.
const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10000);

/// A signer with one key.
const SIGNER: Signer = Signer::with_one_key();


/// Dummy signature used as placeholder.
const DUMMY_SIGNATURE: SignatureEd25519 = signature_ed25519!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");



/// Test minting succeeds and the tokens are owned by the given address and
/// the appropriate events are logged.
#[test]
fn test_minting() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    let token_params = TokenParams {
        amount: TokenAmountU64(100),
        max_supply: TokenAmountU64(1000),
    };

    let mut mint_tokens = BTreeMap::new();
    mint_tokens.insert(TOKEN_1, (
        MetadataUrl {
            url:  TOKEN_1_METADATA.to_string(),
            hash: None,
        }, token_params
    ));

    let mint_params = MintParams {
        owner:      ALICE_ADDR,
        tokens:     mint_tokens,
    };

    // Mint TOKEN_1 to Alice as the owner.
    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.mint".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&mint_params).expect("Mint params"),
        })
        .expect("Mint tokens");

    // Invoke the view entrypoint and check that the tokens are owned by Alice.
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.view".to_string()),
            address:      contract_address,
            message:      OwnedParameter::empty(),
        })
        .expect("Invoke view");

    // Check that the tokens are owned by Alice.
    let rv: ViewState = invoke.parse_return_value().expect("ViewState return value");
    assert_eq!(rv.tokens[..], [TOKEN_0, TOKEN_1]);
    assert_eq!(rv.state, vec![(ALICE_ADDR, ViewAddressState {
        balances:  vec![(TOKEN_0, 100.into()), (TOKEN_1, 200.into())],
        operators: Vec::new(),
    })]);

    // Check that the events are logged.
    let events = update.events().flat_map(|(_addr, events)| events);

    let events: Vec<Cis2Event<ContractTokenId, ContractTokenAmount>> =
        events.map(|e| e.parse().expect("Deserialize event")).collect();

    assert_eq!(events, [
        Cis2Event::Mint(MintEvent {
            token_id: TOKEN_1,
            amount:   TokenAmountU64(100),
            owner:    ALICE_ADDR,
        }),
        Cis2Event::TokenMetadata(TokenMetadataEvent {
            token_id:     TOKEN_1,
            metadata_url: MetadataUrl {
                url:  "https://some.example/token/3F".to_string(),
                hash: None,
            },
        }),
    ]);
}


/// Test regular transfer where sender is the owner.
#[test]
fn test_account_transfer() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Transfer one token from Alice to Bob.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_0,
        amount:   TokenAmountU64(1),
        data:     AdditionalData::empty(),
    }]);

    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect("Transfer tokens");

    // Check that Bob has 1 `TOKEN_0` and Alice has 99. Also check that Alice still
    // has 100 `TOKEN_1`.
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.view".to_string()),
            address:      contract_address,
            message:      OwnedParameter::empty(),
        })
        .expect("Invoke view");
    let rv: ViewState = invoke.parse_return_value().expect("ViewState return value");
    assert_eq!(rv.state, vec![
        (ALICE_ADDR, ViewAddressState {
            balances:  vec![(TOKEN_0, 99.into()), (TOKEN_1, 100.into())],
            operators: Vec::new(),
        }),
        (BOB_ADDR, ViewAddressState {
            balances:  vec![(TOKEN_0, 1.into())],
            operators: Vec::new(),
        }),
    ]);

    // Check that the events are logged.
    let events = update
        .events()
        .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
        .collect::<Vec<Cis2Event<_, _>>>();

    assert_eq!(events, [Cis2Event::Transfer(TransferEvent {
        token_id: TOKEN_0,
        amount:   TokenAmountU64(1),
        from:     ALICE_ADDR,
        to:       BOB_ADDR,
    }),]);
}

/// Test that you can add an operator.
/// Initialize the contract with two tokens owned by Alice.
/// Then add Bob as an operator for Alice.
#[test]
fn test_add_operator() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Add Bob as an operator for Alice.
    let params = UpdateOperatorParams(vec![UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    }]);

    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.updateOperator".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
        })
        .expect("Update operator");

    // Check that an operator event occurred.
    let events = update
        .events()
        .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
        .collect::<Vec<Cis2Event<ContractTokenId, ContractTokenAmount>>>();
    assert_eq!(events, [Cis2Event::UpdateOperator(UpdateOperatorEvent {
        operator: BOB_ADDR,
        owner:    ALICE_ADDR,
        update:   OperatorUpdate::Add,
    }),]);

    // Construct a query parameter to check whether Bob is an operator for Alice.
    let query_params = OperatorOfQueryParams {
        queries: vec![OperatorOfQuery {
            owner:   ALICE_ADDR,
            address: BOB_ADDR,
        }],
    };

    // Invoke the operatorOf entrypoint and check that Bob is an operator for
    // Alice.
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.operatorOf".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&query_params).expect("OperatorOf params"),
        })
        .expect("Invoke opeatorOf");

    let rv: OperatorOfQueryResponse = invoke.parse_return_value().expect("OperatorOf return value");
    assert_eq!(rv, OperatorOfQueryResponse(vec![true]));
}

/// Test that a transfer fails when the sender is neither an operator or the
/// owner. In particular, Bob will attempt to transfer some of Alice's tokens to
/// himself.
#[test]
fn test_unauthorized_sender() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Construct a transfer of `TOKEN_0` from Alice to Bob, which will be submitted
    // by Bob.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_0,
        amount:   TokenAmountU64(1),
        data:     AdditionalData::empty(),
    }]);

    // Notice that Bob is the sender/invoker.
    let update = chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect_err("Transfer tokens");

    // Check that the correct error is returned.
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Unauthorized);
}

/// Test that an operator can make a transfer.
#[test]
fn test_operator_can_transfer() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Add Bob as an operator for Alice.
    let params = UpdateOperatorParams(vec![UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    }]);
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.updateOperator".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
        })
        .expect("Update operator");

    // Let Bob make a transfer to himself on behalf of Alice.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_0,
        amount:   TokenAmountU64(1),
        data:     AdditionalData::empty(),
    }]);

    chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect("Transfer tokens");

    // Check that Bob now has 1 of `TOKEN_0` and Alice has 99. Also check that
    // Alice still has 100 `TOKEN_1`.
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.view".to_string()),
            address:      contract_address,
            message:      OwnedParameter::empty(),
        })
        .expect("Invoke view");
    let rv: ViewState = invoke.parse_return_value().expect("ViewState return value");
    assert_eq!(rv.state, vec![
        (ALICE_ADDR, ViewAddressState {
            balances:  vec![(TOKEN_0, 99.into()), (TOKEN_1, 100.into())],
            operators: vec![BOB_ADDR],
        }),
        (BOB_ADDR, ViewAddressState {
            balances:  vec![(TOKEN_0, 1.into())],
            operators: Vec::new(),
        }),
    ]);
}



/// Test permit mint function. The signature is generated in the test
/// case. ALICE mints tokens to her account.
#[test]
fn test_permit_mint() {
    let (mut chain, keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Check balances in state.
    let balance_of_alice_and_bob = get_balances(&chain, contract_address);

    assert_eq!(balance_of_alice_and_bob.0, [TokenAmountU64(100), TokenAmountU64(0)]);

    let token_params = TokenParams {
        amount: TokenAmountU64(100),
        max_supply: TokenAmountU64(1000),
    };

    let mut mint_tokens = BTreeMap::new();
    mint_tokens.insert(TOKEN_1, (
        MetadataUrl {
            url:  TOKEN_1_METADATA.to_string(),
            hash: None,
        }, token_params
    ));

    // Create input parameters for the `mint` function.
    let payload = MintParams {
        owner:      ALICE_ADDR,
        tokens:     mint_tokens,
    };

    let update =
        permit(&mut chain, contract_address, to_bytes(&payload), "mint".to_string(), keypairs);

    // Check that the correct events occurred.
    let events = update
        .events()
        .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
        .collect::<Vec<Event>>();

    assert_eq!(events, [
        Event::Cis2Event(Cis2Event::Mint(MintEvent {
            token_id: TOKEN_1,
            amount:   TokenAmountU64(100),
            owner:    ALICE_ADDR,
        })),
        Event::Cis2Event(Cis2Event::TokenMetadata(TokenMetadataEvent {
            token_id:     TOKEN_1,
            metadata_url: MetadataUrl {
                url:  "https://some.example/token/3F".to_string(),
                hash: None,
            },
        })),
        Event::Nonce(NonceEvent {
            account: ALICE,
            nonce:   0,
        })
    ]);

    // Check balances in state.
    let balance_of_alice_and_bob = get_balances(&chain, contract_address);

    assert_eq!(balance_of_alice_and_bob.0, [TokenAmountU64(200), TokenAmountU64(0)]);
}




/// Test permit update operator function. The signature is generated in the test
/// case. ALICE adds BOB as an operator.
#[test]
fn test_permit_update_operator() {
    let (mut chain, keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Check operator in state
    let bob_is_operator_of_alice = operator_of(&chain, contract_address);

    assert_eq!(bob_is_operator_of_alice, OperatorOfQueryResponse(vec![false]));

    // Create input parameters for the `permit` updateOperator function.
    let update_operator = UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    };
    let payload = UpdateOperatorParams(vec![update_operator]);

    let update = permit(
        &mut chain,
        contract_address,
        to_bytes(&payload),
        "updateOperator".to_string(),
        keypairs,
    );

    // Check that the correct events occurred.
    let events = update
        .events()
        .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
        .collect::<Vec<Event>>();

    assert_eq!(events, [
        Event::Cis2Event(Cis2Event::UpdateOperator(UpdateOperatorEvent {
            update:   OperatorUpdate::Add,
            owner:    ALICE_ADDR,
            operator: BOB_ADDR,
        })),
        Event::Nonce(NonceEvent {
            account: ALICE,
            nonce:   0,
        })
    ]);

    // Check operator in state
    let bob_is_operator_of_alice = operator_of(&chain, contract_address);

    assert_eq!(bob_is_operator_of_alice, OperatorOfQueryResponse(vec![true]));
}

/// Test permit transfer function. The signature is generated in the test case.
/// TOKEN_1 is transferred from Alice to Bob.
#[test]
fn test_permit_transfer() {
    let (mut chain, keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Check balances in state.
    let balance_of_alice_and_bob = get_balances(&chain, contract_address);

    assert_eq!(balance_of_alice_and_bob.0, [TokenAmountU64(100), TokenAmountU64(0)]);

    // Create input parameters for the `permit` transfer function.
    let transfer = concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::from_account(BOB),
        token_id: TOKEN_1,
        amount:   ContractTokenAmount::from(1),
        data:     AdditionalData::empty(),
    };
    let payload = TransferParams::from(vec![transfer]);

    let update =
        permit(&mut chain, contract_address, to_bytes(&payload), "transfer".to_string(), keypairs);

    // Check that the correct events occurred.
    let events = update
        .events()
        .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
        .collect::<Vec<Event>>();

    assert_eq!(events, [
        Event::Cis2Event(Cis2Event::Transfer(TransferEvent {
            token_id: TOKEN_1,
            amount:   ContractTokenAmount::from(1),
            from:     ALICE_ADDR,
            to:       BOB_ADDR,
        })),
        Event::Nonce(NonceEvent {
            account: ALICE,
            nonce:   0,
        })
    ]);

    // Check balances in state.
    let balance_of_alice_and_bob = get_balances(&chain, contract_address);

    assert_eq!(balance_of_alice_and_bob.0, [TokenAmountU64(99), TokenAmountU64(1)]);
}







/// Test that the pause/unpause entrypoints correctly sets the pause value in
/// the state.
#[test]
fn test_pause_functionality() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Pause the contract.
    chain
        .contract_update(
            SIGNER,
            
            PAUSER,
            PAUSER_ADDR,
            
            Energy::from(10000),
            UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&true).expect("Pause params"),
        })
        .expect("Pause");

    // Check that the contract is now paused.
    assert_eq!(invoke_view(&mut chain, contract_address).paused, true);

    // Unpause the contract.
    chain
        .contract_update(
            SIGNER,
            
            PAUSER,
            PAUSER_ADDR,
            
            Energy::from(10000),
            UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&false).expect("Unpause params"),
        })
        .expect("Unpause");
    // Check that the contract is now unpaused.
    assert_eq!(invoke_view(&mut chain, contract_address).paused, false);
}

/// Test that only the PAUSER can pause/unpause the contract.
#[test]
fn test_pause_unpause_unauthorized() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Pause the contract as Bob, who is not the PAUSER.
    let update = chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&true).expect("Pause params"),
        })
        .expect_err("Pause");

    // Check that the correct error is returned.
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Unauthorized);
}

/// Test that one can NOT call non-admin state-mutative functions (burn,
/// mint, transfer, updateOperator) when the contract is paused.
#[test]
fn test_no_execution_of_state_mutative_functions_when_paused() {
    let (mut chain, _keypairs, contract_address, _module_reference) =
        initialize_chain_and_contract();

    // Pause the contract.
    chain
        .contract_update(
            SIGNER,
            
            PAUSER,
            PAUSER_ADDR,
            
            Energy::from(10000),
            UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&true).expect("Pause params"),
        })
        .expect("Pause");

    // Try to transfer 1 token from Alice to Bob.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_0,
        amount:   TokenAmountU64(1),
        data:     AdditionalData::empty(),
    }]);
    let update_transfer = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect_err("Transfer tokens");
    assert_contract_paused_error(&update_transfer);

    // Try to add Bob as an operator for Alice.
    let params = UpdateOperatorParams(vec![UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    }]);
    let update_operator = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.updateOperator".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
        })
        .expect_err("Update operator");
    assert_contract_paused_error(&update_operator);

    
    // Try to mint tokens.
    let token_params = TokenParams {
        amount: TokenAmountU64(10),
        max_supply: TokenAmountU64(1000),
    };

    let mut mint_tokens = BTreeMap::new();
    mint_tokens.insert(TOKEN_0, (
        MetadataUrl {
            url:  TOKEN_0_METADATA.to_string(),
            hash: None,
        }, token_params
    ));

    let params = MintParams {
        owner:      ALICE_ADDR,
        tokens:     mint_tokens,
    };

    let update_operator = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.mint".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("Mint params"),
        })
        .expect_err("Update operator");
    assert_contract_paused_error(&update_operator);
    

    
}

/// Check that the returned error is `ContractPaused`.
fn assert_contract_paused_error(update: &ContractInvokeError) {
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Custom(CustomContractError::Paused));
}

/// Get the result of the view entrypoint.
fn invoke_view(chain: &mut Chain, contract_address: ContractAddress) -> ViewState {
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.view".to_string()),
            address:      contract_address,
            message:      OwnedParameter::empty(),
        })
        .expect("Invoke view");
    invoke.parse_return_value().expect("Return value")
}



/// Execute a permit function invoke
fn permit(
    chain: &mut Chain,
    contract_address: ContractAddress,
    payload: Vec<u8>,
    entrypoint_name: String,
    keypairs: AccountKeys,
) -> ContractInvokeSuccess {
    // The `viewMessageHash` function uses the same input parameter `PermitParam` as
    // the `permit` function. The `PermitParam` type includes a `signature` and
    // a `signer`. Because these two values (`signature` and `signer`) are not
    // read in the `viewMessageHash` function, any value can be used and we choose
    // to use `DUMMY_SIGNATURE` and `ALICE` in the test case below.
    let signature_map = BTreeMap::from([(0u8, CredentialSignatures {
        sigs: BTreeMap::from([(0u8, concordium_std::Signature::Ed25519(DUMMY_SIGNATURE))]),
    })]);

    let mut param = PermitParam {
        signature: AccountSignatures {
            sigs: signature_map,
        },
        signer:    ALICE,
        message:   PermitMessage {
            timestamp: Timestamp::from_timestamp_millis(10_000_000_000),
            contract_address: ContractAddress::new(0, 0),
            entry_point: OwnedEntrypointName::new_unchecked(entrypoint_name),
            nonce: 0,
            payload,
        },
    };

    // Get the message hash to be signed.
    let invoke = chain
        .contract_invoke(BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            address:      contract_address,
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.viewMessageHash".to_string()),
            message:      OwnedParameter::from_serial(&param)
                .expect("Should be a valid inut parameter"),
        })
        .expect("Should be able to query viewMessageHash");

    let message_hash: HashSha2256 =
        from_bytes(&invoke.return_value).expect("Should return a valid result");

    param.signature = keypairs.sign_message(&to_bytes(&message_hash));

    // Execute permit function.
    chain
        .contract_update(
            Signer::with_one_key(),
            BOB,
            BOB_ADDR,
            Energy::from(10000),
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract_address,
                receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.permit".to_string()),
                message:      OwnedParameter::from_serial(&param)
                    .expect("Should be a valid inut parameter"),
            },
        )
        .expect("Should be able to exit permit token with permit")
}

/// Check if Bob is an operator of Alice.
fn operator_of(chain: &Chain, contract_address: ContractAddress) -> OperatorOfQueryResponse {
    let operator_of_params = OperatorOfQueryParams {
        queries: vec![OperatorOfQuery {
            address: BOB_ADDR,
            owner:   ALICE_ADDR,
        }],
    };

    // Check operator in state
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.operatorOf".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&operator_of_params)
                .expect("OperatorOf params"),
        })
        .expect("Invoke operatorOf");
    let rv: OperatorOfQueryResponse = invoke.parse_return_value().expect("OperatorOf return value");
    rv
}



/// Get the `TOKEN_1` balances for Alice and Bob.
fn get_balances(
    chain: &Chain,
    contract_address: ContractAddress,
) -> ContractBalanceOfQueryResponse {
    let balance_of_params = ContractBalanceOfQueryParams {
        queries: vec![
            BalanceOfQuery {
                token_id: TOKEN_1,
                address:  ALICE_ADDR,
            },
            BalanceOfQuery {
                token_id: TOKEN_1,
                address:  BOB_ADDR,
            },
        ],
    };

    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.balanceOf".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&balance_of_params)
                .expect("BalanceOf params"),
        })
        .expect("Invoke balanceOf");
    let rv: ContractBalanceOfQueryResponse =
        invoke.parse_return_value().expect("BalanceOf return value");
    rv
}


/// Setup chain and contract.
fn initialize_chain_and_contract() -> (Chain, AccountKeys, ContractAddress, ModuleReference) {
    let mut chain = Chain::new();

    let rng = &mut rand::thread_rng();

    let keypairs = AccountKeys::singleton(rng);

    let balance = AccountBalance {
        total:  ACC_INITIAL_BALANCE,
        staked: Amount::zero(),
        locked: Amount::zero(),
    };

    // Create some accounts on the chain.
    chain.create_account(Account::new_with_keys(ALICE, balance, (&keypairs).into()));
    chain.create_account(Account::new(BOB, ACC_INITIAL_BALANCE));
    
    chain.create_account(Account::new(UPGRADER, ACC_INITIAL_BALANCE));
    chain.create_account(Account::new(PAUSER, ACC_INITIAL_BALANCE));
    

    // Load and deploy the module.
    let module = module_load_v1("dist/module.wasm.v1").expect("Module exists");
    let deployment = chain.module_deploy_v1(SIGNER, ALICE, module).expect("Deploy valid module");

    // Init the contract
    let token_params = TokenParams {
        amount: TokenAmountU64(100),
        max_supply: TokenAmountU64(1000),
    };

    let mut premint_tokens = BTreeMap::new();
    premint_tokens.insert(TOKEN_0, (
        MetadataUrl {
            url:  TOKEN_0_METADATA.to_string(),
            hash: None,
        }, token_params
    ));

    let token_params = TokenParams {
        amount: TokenAmountU64(100),
        max_supply: TokenAmountU64(1000),
    };

    premint_tokens.insert(TOKEN_1, (
        MetadataUrl {
            url:  TOKEN_1_METADATA.to_string(),
            hash: None,
        }, token_params
    ));

    let init_params = InitParams {
        premint_tokens: premint_tokens,
    };

    let init = chain
        .contract_init(SIGNER, ALICE, Energy::from(10000), InitContractPayload {
            amount:    Amount::zero(),
            mod_ref:   deployment.module_reference,
            init_name: OwnedContractName::new_unchecked("init_mint_wizard_101101_V3".to_string()),
            param:     OwnedParameter::from_serial(&init_params).expect("Init params"),
        })
        .expect("Initialize contract");

    
    
    
    // Grant PAUSER role
    let grant_role_params = GrantRoleParams {
        address: PAUSER_ADDR,
        role:    Roles::PAUSER,
    };

    let _update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("mint_wizard_101101_V3.grantRole".to_string()),
            address:      init.contract_address,
            message:      OwnedParameter::from_serial(&grant_role_params)
                .expect("GrantRole params"),
        })
        .expect("PAUSER should be granted role");
    
    

    (chain, keypairs, init.contract_address, deployment.module_reference)
}