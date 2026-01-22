mod analytics;
mod topology;
mod terminal;

use super::{NavSection, NavSubItem};

pub fn nav_items(section: NavSection) -> &'static [NavSubItem] {
    match section {
        NavSection::Analytics => &analytics::ANALYTICS_ITEMS,
        NavSection::Topology => &topology::TOPOLOGY_ITEMS,
        NavSection::Terminal => &terminal::TERMINAL_ITEMS,
    }
}
