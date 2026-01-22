//! UI intents emitted by adapters.

use phenome_domain::ActionId;

use super::panel::UiPanelId;
use super::state::UiLogMenuMode;

/// High-level UI intents derived from input events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiIntent {
    TogglePanel(UiPanelId),
    FocusPanel(UiPanelId),
    ScrollPanel { panel: UiPanelId, delta: i16 },
    SelectNext(UiPanelId),
    SelectPrev(UiPanelId),
    ToggleLogMenu(UiLogMenuMode),
    TriggerAction(ActionId),
}
