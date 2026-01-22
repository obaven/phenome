use crate::app::App;

pub(super) fn handle_navbar_click(app: &mut App, column: u16, row: u16) -> bool {
    let pos = (column, row).into();
    if app.ui.nav_flyout_area.contains(pos) {
        for (index, area) in app
            .ui
            .nav_flyout_item_areas
            .iter()
            .take(app.ui.nav_flyout_count)
            .enumerate()
        {
            if area.contains(pos) {
                app.activate_nav_sub(index);
                return true;
            }
        }
    }
    for (index, area) in app.ui.navbar_item_areas.iter().enumerate() {
        if area.contains(pos) {
            let nav = crate::app::NavSection::from_index(index);
            app.set_active_nav(nav);
            return true;
        }
    }
    false
}
