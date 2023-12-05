use super::{
    types::{Network, ReporterRole, ReporterStatus},
    FromPayload,
};
use {
    hapi_core::{client::entities::reporter::Reporter as ReporterPayload, HapiCoreNetwork},
    sea_orm::{entity::prelude::*, Set},
};

// Unlock_timestamp and stake do not correspond to the types of contracts (due to Postgresql restrictions)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "reporter")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub network: Network,
    #[sea_orm(primary_key, auto_increment = false)]
    pub reporter_id: Uuid,
    pub account: String,
    pub role: ReporterRole,
    pub status: ReporterStatus,
    pub name: String,
    pub url: String,
    pub stake: String,
    pub unlock_timestamp: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl FromPayload<ReporterPayload> for ActiveModel {
    fn from(network: &HapiCoreNetwork, payload: ReporterPayload) -> Self {
        Self {
            network: Set(network.clone().into()),
            reporter_id: Set(payload.id.to_owned()),
            account: Set(payload.account.to_owned()),
            role: Set(payload.role.into()),
            status: Set(payload.status.into()),
            name: Set(payload.name.to_owned()),
            url: Set(payload.url.to_owned()),
            stake: Set(payload.stake.to_string()),
            unlock_timestamp: Set(payload.unlock_timestamp.to_string()),
        }
    }
}
