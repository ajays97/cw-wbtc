/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.27.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/
export type ExecuteMsg = {
    transfer_ownership: {
        new_owner_address: string;
    };
} | {
    set_custodian: {
        address: string;
    };
} | {
    add_merchant: {
        address: string;
    };
} | {
    remove_merchant: {
        address: string;
    };
} | {
    set_custodian_deposit_address: {
        deposit_address: string;
        merchant: string;
    };
} | {
    set_merchant_deposit_address: {
        deposit_address: string;
    };
} | {
    add_mint_request: {
        amount: Uint128;
        deposit_address: string;
        tx_id: string;
    };
} | {
    cancel_mint_request: {
        request_hash: string;
    };
} | {
    confirm_mint_request: {
        request_hash: string;
    };
} | {
    reject_mint_request: {
        request_hash: string;
    };
} | {
    burn: {
        amount: Uint128;
    };
} | {
    confirm_burn_request: {
        request_hash: string;
        tx_id: string;
    };
} | {
    pause: {};
} | {
    unpause: {};
};
export type Uint128 = string;
export interface InstantiateMsg {
    owner: string;
}
export type MigrateMsg = string;
export type QueryMsg = {
    get_mint_request: {
        nonce: Uint64;
    };
} | {
    get_mint_requests_length: {};
} | {
    get_burn_request: {
        nonce: Uint64;
    };
} | {
    get_burn_requests_length: {};
} | {
    get_token_denom: {};
} | {
    is_merchant: {
        address: string;
    };
} | {
    is_custodian: {
        address: string;
    };
} | {
    get_custodian: {};
} | {
    get_owner: {};
} | {
    is_owner: {
        address: string;
    };
};
export type Uint64 = string;
export type Timestamp = Uint64;
export interface GetBurnRequestResponse {
    amount: Uint128;
    deposit_address: string;
    request_hash: string;
    request_nonce: Uint64;
    requester: string;
    status: string;
    timestamp: Timestamp;
    tx_id: string;
}
export interface GetBurnRequestsLengthResponse {
    length: Uint64;
}
export interface GetCustodianResponse {
    address: string;
}
export interface GetMintRequestResponse {
    amount: Uint64;
    deposit_address: string;
    request_hash: string;
    request_nonce: Uint64;
    requester: string;
    status: string;
    timestamp: Timestamp;
    tx_id: string;
}
export interface GetMintRequestsLengthResponse {
    length: Uint64;
}
export interface GetOwnerResponse {
    address: string;
}
export interface GetTokenDenomResponse {
    denom: string;
}
export interface IsCustodianResponse {
    is_custodian: boolean;
}
export interface IsMerchantResponse {
    is_merchant: boolean;
}
export interface IsOwnerResponse {
    is_owner: boolean;
}
//# sourceMappingURL=WbtcController.types.d.ts.map