//! ALPG000 program-options action: the catalogue programs the audit list can be filtered by

use super::alpg000_executor::Alpg000CommandExecutor;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::ProgramOptionDto;

impl Alpg000CommandExecutor {
    pub(super) async fn list_program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error> {
        match &self.programs {
            Some(source) => {
                use application_core::traits::{cache_ns, cache_ttl};
                let cache = self.caches.ns(cache_ns::PROGRAMS);
                let key = cache.key("alpg000_program_options", &());
                cache.through_key(key, cache_ttl::PROGRAMS, || source.list_program_options()).await
            }
            None => Ok(Vec::new()),
        }
    }
}
