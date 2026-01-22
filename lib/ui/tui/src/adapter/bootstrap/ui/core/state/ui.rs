use primer::application::flows::reconcile::visualize::LayerType;
use std::collections::{HashSet, VecDeque};

use phenome_domain::Event;

use super::{FocusTarget, MenuState};

#[derive(Default)]
pub struct BootstrapUiState {
    pub show_summary: bool,
    pub completed_seen: bool,
    pub show_logs: bool,
    pub focus: FocusTarget,
    pub tree_selected: usize,
    pub tree_scroll: usize,
    pub collapsed_layers: HashSet<LayerType>,
    pub status_selected: usize,
    pub status_scroll: usize,
    pub expanded_components: HashSet<String>,
    pub menu_state: MenuState,
    pub paused: bool,
    pub log_events: VecDeque<Event>,
    pub log_scroll: usize,
    pub log_view_height: usize,
}
