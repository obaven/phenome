#[macro_export]
macro_rules! grid_spec {
    (rows: [$row:expr $(, $rows:expr)* $(,)?], cols: [$col:expr $(, $cols:expr)* $(,)?], slots: [$($slot:expr),* $(,)?]) => {{
        let rows = vec![$row $(, $rows)*];
        let cols = vec![$col $(, $cols)*];
        debug_assert!(!rows.is_empty(), "grid_spec! requires at least one row");
        debug_assert!(!cols.is_empty(), "grid_spec! requires at least one col");
        $crate::layout::GridSpec::new(rows, cols)
            .with_slots(vec![$($slot),*])
    }};
    (rows: [$row:expr $(, $rows:expr)* $(,)?], cols: [$col:expr $(, $cols:expr)* $(,)?], slots: $slots:expr $(,)?) => {{
        let rows = vec![$row $(, $rows)*];
        let cols = vec![$col $(, $cols)*];
        debug_assert!(!rows.is_empty(), "grid_spec! requires at least one row");
        debug_assert!(!cols.is_empty(), "grid_spec! requires at least one col");
        $crate::layout::GridSpec::new(rows, cols)
            .with_slots($slots)
    }};
}

#[macro_export]
macro_rules! grid_slot {
    ($id:expr, $row:expr, $col:expr $(, $($rest:tt)*)? ) => {{
        let slot = $crate::layout::GridSlot::new($id, $row, $col);
        $crate::grid_slot_opts!(slot $(, $($rest)*)?)
    }};
}

#[macro_export]
macro_rules! grid_slots {
    ($($slot:expr),* $(,)?) => {{
        let mut slots = Vec::new();
        $(slots.push($slot);)*
        slots
    }};
}

#[macro_export]
macro_rules! grid_slot_opts {
    ($slot:expr) => { $slot };
    ($slot:expr, span: ($r:expr, $c:expr) $(, $($rest:tt)*)? ) => {
        $crate::grid_slot_opts!($slot.span($r, $c) $(, $($rest)*)?)
    };
    ($slot:expr, hidden $(, $($rest:tt)*)? ) => {
        $crate::grid_slot_opts!($slot.hidden() $(, $($rest)*)?)
    };
    ($slot:expr, movable: $val:expr $(, $($rest:tt)*)? ) => {
        $crate::grid_slot_opts!($slot.movable($val) $(, $($rest)*)?)
    };
    ($slot:expr, min: ($w:expr, $h:expr) $(, $($rest:tt)*)? ) => {
        $crate::grid_slot_opts!($slot.with_min_size($w, $h) $(, $($rest)*)?)
    };
    ($slot:expr, max: ($w:expr, $h:expr) $(, $($rest:tt)*)? ) => {
        $crate::grid_slot_opts!($slot.with_max_size($w, $h) $(, $($rest)*)?)
    };
    ($slot:expr, offset: ($x:expr, $y:expr) $(, $($rest:tt)*)? ) => {
        $crate::grid_slot_opts!($slot.offset($x, $y) $(, $($rest)*)?)
    };
}

#[cfg(test)]
mod tests {
    use crate::layout::TrackSize;

    #[test]
    fn builds_spec_with_macros() {
        let spec = crate::grid_spec!(
            rows: [TrackSize::Fixed(2), TrackSize::Fixed(2)],
            cols: [TrackSize::Fixed(3)],
            slots: [
                crate::grid_slot!("header", 0, 0, span: (1, 1)),
                crate::grid_slot!("body", 1, 0, movable: true, min: (2, 2)),
            ]
        );

        assert_eq!(spec.rows.len(), 2);
        assert_eq!(spec.cols.len(), 1);
        assert_eq!(spec.slots.len(), 2);
        assert_eq!(spec.slots[0].id.as_str(), "header");
        assert!(spec.slots[1].movable);
        assert_eq!(spec.slots[1].min_width, Some(2));
    }

    #[test]
    fn composes_slot_lists() {
        let primary = crate::grid_slots!(
            crate::grid_slot!("header", 0, 0, span: (1, 1)),
            crate::grid_slot!("body", 1, 0),
        );
        let mut slots = primary;
        slots.extend(crate::grid_slots!(
            crate::grid_slot!("footer", 2, 0, min: (2, 2))
        ));
        let spec = crate::grid_spec!(
            rows: [TrackSize::Fixed(1), TrackSize::Fixed(1), TrackSize::Fixed(1)],
            cols: [TrackSize::Fixed(3)],
            slots: slots
        );

        assert_eq!(spec.slots.len(), 3);
        assert_eq!(spec.slots[2].id.as_str(), "footer");
        assert_eq!(spec.rows.len(), 3);
        assert_eq!(spec.cols.len(), 1);
    }
}
