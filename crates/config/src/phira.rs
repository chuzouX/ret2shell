use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::traits::Merge;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, FromJsonQueryResult, Default)]
pub struct Config {
  pub base_url: String,
}

impl Merge for Option<Config> {
  fn merge(self, other: Self) -> Self {
    other.or(self)
  }
}
