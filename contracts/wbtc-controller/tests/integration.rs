use std::path::PathBuf;

use cosmwasm_std::{Coin, Uint128};

use osmosis_test_tube::{
    cosmrs::proto::{
        cosmos::bank::v1beta1::{
            DenomUnit, Metadata, QueryBalanceRequest, QueryBalanceResponse,
            QueryDenomMetadataRequest, QueryDenomMetadataResponse, QuerySupplyOfRequest,
            QuerySupplyOfResponse,
        },
        cosmwasm::wasm::v1::MsgExecuteContractResponse,
    },
    Account, Bank, Module, OsmosisTestApp, Runner, RunnerError, RunnerExecuteResult, RunnerResult,
    SigningAccount, Wasm,
};
use serde::de::DeserializeOwned;
use wbtc_controller::{
    msg::{
        ExecuteMsg, GetBurnRequestByHashResponse, GetBurnRequestByNonceResponse,
        GetMintRequestByHashResponse, GetMintRequestByNonceResponse, GetTokenDenomResponse,
        InstantiateMsg, ListBurnRequestsResponse, ListMintRequestsResponse, QueryMsg,
    },
    BurnRequestStatus, MintRequestStatus, TxId,
};

pub struct WBTC<'a> {
    app: &'a OsmosisTestApp,
    pub code_id: u64,
    pub contract_addr: String,
}

impl<'a> WBTC<'a> {
    pub fn deploy(
        app: &'a OsmosisTestApp,
        msg: &InstantiateMsg,
        funds: &[Coin],
        signer: &SigningAccount,
    ) -> Result<Self, RunnerError> {
        let wasm = Wasm::new(app);

        let code_id = wasm
            .store_code(&Self::get_wasm_byte_code(), None, signer)?
            .data
            .code_id;
        let contract_addr = wasm
            .instantiate(code_id, msg, None, None, funds, signer)?
            .data
            .address;

        Ok(Self {
            app,
            code_id,
            contract_addr,
        })
    }

    pub fn execute(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        let wasm = Wasm::new(self.app);
        wasm.execute(&self.contract_addr, msg, funds, signer)
    }

    pub fn query<Res>(&self, msg: &QueryMsg) -> RunnerResult<Res>
    where
        Res: ?Sized + DeserializeOwned,
    {
        let wasm = Wasm::new(self.app);
        wasm.query::<QueryMsg, Res>(&self.contract_addr, msg)
    }

    fn get_wasm_byte_code() -> Vec<u8> {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        std::fs::read(
            manifest_path
                .join("..")
                .join("..")
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
                .join("wbtc_controller.wasm"),
        )
        .unwrap()
    }
}

#[test]
fn test_set_denom_metadata() {
    let app = OsmosisTestApp::default();

    let accs = app
        .init_accounts(&[Coin::new(100_000_000_000, "uosmo")], 2)
        .unwrap();
    let owner = &accs[0];
    let other = &accs[1];

    let wbtc = WBTC::deploy(
        &app,
        &InstantiateMsg {
            owner: owner.address(),
            subdenom: "wbtc".to_string(),
        },
        &[Coin::new(10000000, "uosmo")],
        &owner,
    )
    .unwrap();

    // get token denom
    let GetTokenDenomResponse { denom } = wbtc
        .query::<GetTokenDenomResponse>(&QueryMsg::GetTokenDenom {})
        .unwrap();

    assert_eq!(
        app.query::<QueryDenomMetadataRequest, QueryDenomMetadataResponse>(
            "/cosmos.bank.v1beta1.Query/DenomMetadata",
            &QueryDenomMetadataRequest {
                denom: denom.clone()
            }
        )
        .unwrap()
        .metadata
        .unwrap(),
        Metadata {
            description: "".to_string(),
            denom_units: vec![vec![
                // must start with `denom` with exponent 0
                DenomUnit {
                    denom: denom.clone(),
                    exponent: 0,
                    aliases: vec![],
                }
            ],]
            .concat(),
            base: denom.clone(),
            display: "".to_string(),
            name: "".to_string(),
            symbol: "".to_string(),
        }
    );

    let updated_metadata = osmosis_std::types::cosmos::bank::v1beta1::Metadata {
        description: "Tokenfactory-based token backed 1:1 with Bitcoin. Completely transparent. 100% verifiable. Community led."
            .to_string(),
        denom_units: vec![vec![
            // must start with `denom` with exponent 0
            osmosis_std::types::cosmos::bank::v1beta1::DenomUnit {
                denom: denom.clone(),
                exponent: 0,
                aliases: vec!["uwbtc".to_string()],
            },
            osmosis_std::types::cosmos::bank::v1beta1::DenomUnit {
                denom: "wbtc".to_string(),
                exponent: 16,
                aliases: vec![],
            }
        ]]
        .concat(),
        base: denom.clone(),
        display: "wbtc".to_string(),
        name: "Wrapped Bitcoin".to_string(),
        symbol: "WBTC".to_string(),
    };

    // set denom metadata by non owner should fail
    let err = wbtc
        .execute(
            &ExecuteMsg::SetDenomMetadata {
                metadata: updated_metadata.clone(),
            },
            &[],
            &other,
        )
        .unwrap_err();

    assert_eq!(
        err,
        RunnerError::ExecuteError {
            msg: "failed to execute message; message index: 0: Unauthorized: execute wasm contract failed".to_string()
        }
    );

    // set denom metadata by owner should succeed
    wbtc.execute(
        &ExecuteMsg::SetDenomMetadata {
            metadata: updated_metadata.clone(),
        },
        &[],
        &owner,
    )
    .unwrap();

    assert_eq!(
        app.query::<QueryDenomMetadataRequest, QueryDenomMetadataResponse>(
            "/cosmos.bank.v1beta1.Query/DenomMetadata",
            &QueryDenomMetadataRequest {
                denom: denom.clone()
            }
        )
        .unwrap()
        .metadata
        .unwrap(),
        Metadata {
            description: updated_metadata.description,
            denom_units: vec![
                // must start with `denom` with exponent 0
                DenomUnit {
                    denom: denom.clone(),
                    exponent: 0,
                    aliases: vec!["uwbtc".to_string()],
                },
                DenomUnit {
                    denom: "wbtc".to_string(),
                    exponent: 16,
                    aliases: vec![],
                }
            ],
            base: updated_metadata.base,
            display: updated_metadata.display,
            name: updated_metadata.name,
            symbol: updated_metadata.symbol,
        }
    );
}

#[test]
fn test_mint_and_burn() {
    let app = OsmosisTestApp::default();
    let bank = Bank::new(&app);

    let accs = app
        .init_accounts(&[Coin::new(100_000_000_000, "uosmo")], 4)
        .unwrap();
    let owner = &accs[0];
    let custodian = &accs[1];
    let merchant = &accs[2];

    let wbtc = WBTC::deploy(
        &app,
        &InstantiateMsg {
            owner: owner.address(),
            subdenom: "wbtc".to_string(),
        },
        &[Coin::new(10000000, "uosmo")],
        &owner,
    )
    .unwrap();

    // get token denom
    let GetTokenDenomResponse { denom } = wbtc
        .query::<GetTokenDenomResponse>(&QueryMsg::GetTokenDenom {})
        .unwrap();

    assert_eq!(
        app.query::<QuerySupplyOfRequest, QuerySupplyOfResponse>(
            "/cosmos.bank.v1beta1.Query/SupplyOf",
            &QuerySupplyOfRequest {
                denom: denom.clone(),
            }
        )
        .unwrap()
        .amount
        .unwrap()
        .amount,
        "0"
    );

    // set custodian
    wbtc.execute(
        &ExecuteMsg::SetCustodian {
            address: custodian.address(),
        },
        &[],
        &owner,
    )
    .unwrap();

    // setup merchants

    // add merchant
    wbtc.execute(
        &ExecuteMsg::AddMerchant {
            address: merchant.address(),
        },
        &[],
        &owner,
    )
    .unwrap();

    // set custodian deposit address
    wbtc.execute(
        &ExecuteMsg::SetCustodianDepositAddress {
            merchant: merchant.address(),
            deposit_address: format!("bc1{}", merchant.address()),
        },
        &[],
        &custodian,
    )
    .unwrap();

    // set merchant deposit address
    wbtc.execute(
        &ExecuteMsg::SetMerchantDepositAddress {
            deposit_address: format!("bc1{}", merchant.address()),
        },
        &[],
        &merchant,
    )
    .unwrap();

    // issue mint request
    let amount = Uint128::from(100000000u128);
    wbtc.execute(
        &ExecuteMsg::IssueMintRequest {
            amount,
            tx_id: "tx_id_1".to_string(),
            deposit_address: format!("bc1{}", merchant.address()),
        },
        &[],
        &merchant,
    )
    .unwrap();

    // check mint request
    let res: ListMintRequestsResponse = wbtc
        .query(&QueryMsg::ListMintRequests {
            limit: None,
            start_after_nonce: None,
            status: None,
        })
        .unwrap();

    let req = &res.requests[0].clone();

    let req_by_hash = wbtc
        .query::<GetMintRequestByHashResponse>(&QueryMsg::GetMintRequestByHash {
            request_hash: req.request_hash.clone(),
        })
        .unwrap()
        .request;

    let res_by_nonce = wbtc
        .query::<GetMintRequestByNonceResponse>(&QueryMsg::GetMintRequestByNonce {
            nonce: req.request.data.nonce,
        })
        .unwrap()
        .request;

    assert_eq!(req_by_hash, req.request);
    assert_eq!(res_by_nonce, req.request);
    assert_eq!(req.request.status, MintRequestStatus::Pending);

    // approve mint request
    wbtc.execute(
        &ExecuteMsg::ApproveMintRequest {
            request_hash: req.request_hash.clone(),
        },
        &[],
        &custodian,
    )
    .unwrap();

    assert_eq!(
        wbtc.query::<GetMintRequestByHashResponse>(&QueryMsg::GetMintRequestByHash {
            request_hash: req.request_hash.clone(),
        })
        .unwrap()
        .request
        .status,
        MintRequestStatus::Approved
    );

    // check balance
    let QueryBalanceResponse { balance } = bank
        .query_balance(&QueryBalanceRequest {
            address: merchant.address(),
            denom: denom.clone(),
        })
        .unwrap();

    assert_eq!(balance.unwrap().amount, amount.to_string());

    // check total supply
    assert_eq!(
        app.query::<QuerySupplyOfRequest, QuerySupplyOfResponse>(
            "/cosmos.bank.v1beta1.Query/SupplyOf",
            &QuerySupplyOfRequest {
                denom: denom.clone(),
            }
        )
        .unwrap()
        .amount
        .unwrap()
        .amount,
        amount.to_string()
    );

    // issue burn request
    wbtc.execute(
        &ExecuteMsg::Burn { amount },
        &[Coin::new(amount.u128(), denom.clone())],
        &merchant,
    )
    .unwrap();

    // check burn request
    let res: ListBurnRequestsResponse = wbtc
        .query(&QueryMsg::ListBurnRequests {
            limit: None,
            start_after_nonce: None,
            status: None,
        })
        .unwrap();

    let req = &res.requests[0].clone();

    let req_by_hash = wbtc
        .query::<GetBurnRequestByHashResponse>(&QueryMsg::GetBurnRequestByHash {
            request_hash: req.request_hash.clone(),
        })
        .unwrap()
        .request;

    let res_by_nonce = wbtc
        .query::<GetBurnRequestByNonceResponse>(&QueryMsg::GetBurnRequestByNonce {
            nonce: req.request.data.nonce,
        })
        .unwrap()
        .request;

    assert_eq!(req_by_hash, req.request);
    assert_eq!(res_by_nonce, req.request);
    assert_eq!(req.request.status, BurnRequestStatus::Executed);
    assert_eq!(req.request.data.tx_id, TxId::Pending);

    // check supply
    assert_eq!(
        app.query::<QuerySupplyOfRequest, QuerySupplyOfResponse>(
            "/cosmos.bank.v1beta1.Query/SupplyOf",
            &QuerySupplyOfRequest {
                denom: denom.clone(),
            }
        )
        .unwrap()
        .amount
        .unwrap()
        .amount,
        "0"
    );

    // check balance
    let QueryBalanceResponse { balance } = bank
        .query_balance(&QueryBalanceRequest {
            address: merchant.address(),
            denom: denom.clone(),
        })
        .unwrap();

    assert_eq!(balance.unwrap().amount, "0");

    // confirm burn request
    wbtc.execute(
        &ExecuteMsg::ConfirmBurnRequest {
            request_hash: req.request_hash.clone(),
            tx_id: "tx_id_2".to_string(),
        },
        &[],
        &custodian,
    )
    .unwrap();

    assert_eq!(
        wbtc.query::<GetBurnRequestByHashResponse>(&QueryMsg::GetBurnRequestByHash {
            request_hash: req.request_hash.clone(),
        })
        .unwrap()
        .request
        .data
        .tx_id,
        TxId::Confirmed("tx_id_2".to_string())
    );
}
