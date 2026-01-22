use super::{NavAction, NavSection, NavSubItem, NavView, nav_items};
use crate::app::App;

impl App {
    pub fn active_nav(&self) -> NavSection {
        self.active_nav
    }

    pub fn active_view(&self) -> NavView {
        self.active_view
    }

    pub fn set_active_nav(&mut self, nav: NavSection) {
        self.active_nav = nav;
        let items = nav_items(nav);
        if items.is_empty() {
            return;
        }
        let index = self.nav_sub_index[nav.index()].min(items.len().saturating_sub(1));
        self.nav_sub_index[nav.index()] = index;
        self.active_view = items[index].view;
    }

    pub fn next_nav(&mut self) {
        self.active_nav = self.active_nav.next();
        self.set_active_nav(self.active_nav);
    }

    pub fn prev_nav(&mut self) {
        self.active_nav = self.active_nav.prev();
        self.set_active_nav(self.active_nav);
    }

    pub fn nav_sub_index(&self, section: NavSection) -> usize {
        self.nav_sub_index[section.index()]
    }

    pub fn set_nav_sub_index(&mut self, index: usize) {
        let section = self.active_nav;
        let items = nav_items(section);
        if items.is_empty() {
            return;
        }
        let clamped = index.min(items.len().saturating_sub(1));
        self.nav_sub_index[section.index()] = clamped;
        self.active_view = items[clamped].view;
    }

    pub fn activate_nav_sub(&mut self, index: usize) {
        self.set_nav_sub_index(index);
        let section = self.active_nav;
        let items = nav_items(section);
        if let Some(item) = items.get(self.nav_sub_index(section)) {
            self.execute_nav_action(item.action);
        }
    }

    pub fn next_nav_sub(&mut self) {
        let section = self.active_nav;
        let items = nav_items(section);
        if items.is_empty() {
            return;
        }
        let current = self.nav_sub_index(section);
        let next = (current + 1) % items.len();
        self.set_nav_sub_index(next);
    }

    pub fn prev_nav_sub(&mut self) {
        let section = self.active_nav;
        let items = nav_items(section);
        if items.is_empty() {
            return;
        }
        let current = self.nav_sub_index(section);
        let next = (current + items.len() - 1) % items.len();
        self.set_nav_sub_index(next);
    }

    pub fn active_subitem(&self) -> Option<NavSubItem> {
        let items = nav_items(self.active_nav);
        items.get(self.nav_sub_index(self.active_nav)).copied()
    }

    fn execute_nav_action(&mut self, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::RefreshSnapshot => {
                self.runtime.refresh_snapshot();
                self.refresh_log_cache(true);
            }
            NavAction::ToggleNotifications => {
                self.toggle_notifications_panel();
            }
            NavAction::ToggleWatch => {
                self.ui.auto_refresh = !self.ui.auto_refresh;
            }
            NavAction::CycleLogFilter => {
                self.ui.log_config.filter = self.ui.log_config.filter.next();
                self.refresh_log_cache(true);
            }
            NavAction::NextLogInterval => {
                self.cycle_log_interval();
                self.refresh_log_cache(true);
            }
        }
    }
}
