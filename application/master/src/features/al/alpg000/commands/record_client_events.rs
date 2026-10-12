//! ALPG000 client-event intake action

use super::alpg000_executor::Alpg000CommandExecutor;
use super::client_event_rules::normalize_client_events;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{ClientActor, ClientEventCommand};

impl Alpg000CommandExecutor {
    pub(super) fn record_client_events_action(
        &self,
        actor: ClientActor,
        events: Vec<ClientEventCommand>,
    ) -> Result<usize, Alpg000Error> {
        let events = normalize_client_events(&actor, events)?;
        let accepted = events.len();
        if accepted > 0 {
            // Fire and forget: the port drops events when the store is down
            self.audit.record(events);
        }
        Ok(accepted)
    }
}
