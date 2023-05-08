use cosmwasm_std::{Addr, Deps, DepsMut, Event, MessageInfo, Response, StdError};
use cw_storage_plus::Item;

use crate::ContractError;

use super::{allow_only, Role};

const CUSTODIAN: Item<Addr> = Item::new("custodian");

pub fn set_custodian(
    deps: DepsMut,
    info: &MessageInfo,
    address: &str,
) -> Result<Response, ContractError> {
    allow_only(&[Role::Owner], &info.sender, deps.as_ref())?;

    CUSTODIAN.save(deps.storage, &deps.api.addr_validate(address)?)?;

    let event = Event::new("set_custodian").add_attribute("address", address);
    Ok(Response::new().add_event(event))
}

pub fn is_custodian(deps: Deps, address: &Addr) -> Result<bool, StdError> {
    let custodian = CUSTODIAN.load(deps.storage)?;

    Ok(custodian == address)
}

pub fn get_custodian(deps: Deps) -> Result<Addr, StdError> {
    CUSTODIAN.load(deps.storage)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};

    use crate::auth::owner::initialize_owner;

    use super::*;

    #[test]
    fn test_manage_custodian() {
        let mut deps = mock_dependencies();
        let owner = "osmo1owner";
        let non_owner = "osmo1nonowner";
        let custodian_address = "osmo1custodian";
        let non_custodian_address = "osmo1noncustodian";

        // setup
        initialize_owner(deps.as_mut(), owner).unwrap();

        // check before set will fail
        let err = is_custodian(deps.as_ref(), &Addr::unchecked(custodian_address)).unwrap_err();
        assert_eq!(err, StdError::not_found("cosmwasm_std::addresses::Addr"));

        let err = get_custodian(deps.as_ref()).unwrap_err();
        assert_eq!(err, StdError::not_found("cosmwasm_std::addresses::Addr"));

        // set custodian by non owner should fail
        let err = set_custodian(
            deps.as_mut(),
            &mock_info(non_owner, &[]),
            &custodian_address,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // set custodian
        assert_eq!(
            set_custodian(deps.as_mut(), &mock_info(owner, &[]), &custodian_address)
                .unwrap()
                .events,
            vec![Event::new("set_custodian").add_attribute("address", custodian_address.clone())]
        );

        // check after set will pass
        assert_eq!(get_custodian(deps.as_ref()).unwrap(), custodian_address);
        assert_eq!(
            is_custodian(deps.as_ref(), &Addr::unchecked(custodian_address)).unwrap(),
            true
        );
        assert_eq!(
            is_custodian(deps.as_ref(), &Addr::unchecked(non_custodian_address)).unwrap(),
            false
        );
    }
}
