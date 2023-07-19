use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
}

#[cw_serde]
pub struct InputVariable {
    pub key: String,
    pub value: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CosmosMsg)]
    GetHydratedMsg {
        msg_template: String,
        msg_params: Vec<InputVariable>,
    },
}
