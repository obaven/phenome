//! Layout policy orchestration.

use std::sync::{Arc, RwLock};

use ratatui::layout::Rect;

use crate::layout::{GridSpec, SlotId};

use super::{GroupPolicy, SlotOverride, SlotPolicy};
use super::state::{apply_override, PolicyState};

/// Thread-safe policy container for grid layouts.
#[derive(Clone)]
pub struct LayoutPolicy {
    state: Arc<RwLock<PolicyState>>,
}

impl LayoutPolicy {
    /// Create a new policy collection.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PolicyState::default())),
        }
    }

    /// Attach a policy for a slot id.
    pub fn set_policy(&self, slot: impl Into<SlotId>, policy: SlotPolicy) {
        if let Ok(mut guard) = self.state.write() {
            guard.policies.insert(slot.into(), policy);
        }
    }

    /// Register a group of slots.
    pub fn set_group(&self, group: GroupPolicy) {
        if let Ok(mut guard) = self.state.write() {
            if let Some(existing) = guard.groups.iter_mut().find(|rule| rule.name == group.name) {
                *existing = group;
            } else {
                guard.groups.push(group);
            }
        }
    }

    /// Add a collapse rule from a trigger slot to targets.
    pub fn add_collapse_rule(&self, trigger: impl Into<SlotId>, targets: Vec<SlotId>) {
        let trigger = trigger.into();
        if let Ok(mut guard) = self.state.write() {
            guard
                .collapse_rules
                .entry(trigger)
                .or_default()
                .extend(targets);
        }
    }

    /// Pin or unpin a slot to avoid automatic collapse.
    pub fn set_pinned(&self, slot: impl Into<SlotId>, pinned: bool) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            if pinned {
                guard.pinned.insert(slot);
            } else {
                guard.pinned.remove(&slot);
            }
        }
    }

    /// Apply an explicit override to a slot.
    pub fn set_override(&self, slot: impl Into<SlotId>, override_data: SlotOverride) {
        if let Ok(mut guard) = self.state.write() {
            guard.overrides.insert(slot.into(), override_data);
        }
    }

    /// Clear overrides for a slot.
    pub fn clear_override(&self, slot: &SlotId) {
        if let Ok(mut guard) = self.state.write() {
            guard.overrides.remove(slot);
        }
    }

    /// Explicitly set visibility for a slot.
    pub fn set_visibility(&self, slot: impl Into<SlotId>, visible: bool) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            let entry = guard.overrides.entry(slot).or_default();
            entry.visible = Some(visible);
        }
    }

    /// Resolve visibility for a slot id.
    pub fn visibility_for(&self, slot: &str) -> Option<bool> {
        let guard = self.state.read().ok()?;
        guard.overrides.get(slot).and_then(|entry| entry.visible)
    }

    /// Explicitly set collapsed state for a slot.
    pub fn set_collapsed(&self, slot: impl Into<SlotId>, collapsed: bool) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            let entry = guard.overrides.entry(slot).or_default();
            entry.collapsed = Some(collapsed);
        }
    }

    /// Resolve collapsed state for a slot id.
    pub fn collapsed_for(&self, slot: &str) -> Option<bool> {
        let guard = self.state.read().ok()?;
        guard.overrides.get(slot).and_then(|entry| entry.collapsed)
    }

    /// Set a row/column override for a slot.
    pub fn set_position(&self, slot: impl Into<SlotId>, row: usize, col: usize) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            let entry = guard.overrides.entry(slot).or_default();
            entry.row = Some(row);
            entry.col = Some(col);
        }
    }

    /// Set the row/column span override for a slot.
    pub fn set_span(&self, slot: impl Into<SlotId>, row_span: usize, col_span: usize) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            let entry = guard.overrides.entry(slot).or_default();
            entry.row_span = Some(row_span.max(1));
            entry.col_span = Some(col_span.max(1));
        }
    }

    /// Request that a slot be opened and enforce group policies.
    pub fn request_open(&self, slot: impl Into<SlotId>, area: Rect) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            guard.bump_usage(&slot);
            guard.set_collapsed(&slot, false);
            guard.apply_collapse_rules(&slot);
            guard.apply_group_exclusivity(&slot);
            guard.apply_low_space(area, Some(&slot));
        }
    }

    /// Request that a slot be collapsed.
    pub fn request_close(&self, slot: impl Into<SlotId>) {
        let slot = slot.into();
        if let Ok(mut guard) = self.state.write() {
            guard.set_collapsed(&slot, true);
        }
    }

    /// Apply policies and overrides to a grid spec.
    pub fn apply(&self, spec: &GridSpec, _area: Rect) -> GridSpec {
        let guard = match self.state.read() {
            Ok(guard) => guard,
            Err(_) => return spec.clone(),
        };
        let mut next = spec.clone();
        for slot in next.slots.iter_mut() {
            if let Some(policy) = guard.policies.get(&slot.id) {
                slot.movable = policy.movable;
                slot.min_width = policy.min_width;
                slot.min_height = policy.min_height;
                slot.max_width = policy.max_width;
                slot.max_height = policy.max_height;
            }
            if let Some(override_data) = guard.overrides.get(&slot.id) {
                apply_override(slot, override_data);
            }
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{GridSlot, GroupPolicy, PanelPriority, SlotPolicy, TrackSize};

    #[test]
    fn applies_visibility_override() {
        let policy = LayoutPolicy::new();
        policy.set_visibility("slot", false);

        let spec = GridSpec::new(vec![TrackSize::Fixed(2)], vec![TrackSize::Fixed(2)])
            .with_slots(vec![GridSlot::new("slot", 0, 0)]);
        let updated = policy.apply(&spec, Rect::new(0, 0, 4, 4));
        assert_eq!(updated.slots.len(), 1);
        assert!(!updated.slots[0].visible);
    }

    #[test]
    fn applies_position_and_span_override() {
        let policy = LayoutPolicy::new();
        policy.set_position("slot", 1, 2);
        policy.set_span("slot", 2, 3);

        let spec = GridSpec::new(
            vec![TrackSize::Fixed(2), TrackSize::Fixed(2), TrackSize::Fixed(2)],
            vec![TrackSize::Fixed(2), TrackSize::Fixed(2), TrackSize::Fixed(2)],
        )
        .with_slots(vec![GridSlot::new("slot", 0, 0)]);
        let updated = policy.apply(&spec, Rect::new(0, 0, 6, 6));
        assert_eq!(updated.slots[0].row, 1);
        assert_eq!(updated.slots[0].col, 2);
        assert_eq!(updated.slots[0].row_span, 2);
        assert_eq!(updated.slots[0].col_span, 3);
    }

    #[test]
    fn applies_policy_attributes() {
        let policy = LayoutPolicy::new();
        policy.set_policy(
            "slot",
            SlotPolicy::new(PanelPriority::High)
                .movable(true)
                .min_size(4, 3)
                .max_size(8, 6),
        );

        let spec = GridSpec::new(vec![TrackSize::Fixed(10)], vec![TrackSize::Fixed(10)])
            .with_slots(vec![GridSlot::new("slot", 0, 0)]);
        let updated = policy.apply(&spec, Rect::new(0, 0, 10, 10));
        let slot = &updated.slots[0];
        assert!(slot.movable);
        assert_eq!(slot.min_width, Some(4));
        assert_eq!(slot.min_height, Some(3));
        assert_eq!(slot.max_width, Some(8));
        assert_eq!(slot.max_height, Some(6));
    }

    #[test]
    fn request_open_collapses_exclusive_group() {
        let policy = LayoutPolicy::new();
        policy.set_group(
            GroupPolicy::new("exclusive", vec!["a".into(), "b".into()]).exclusive(true),
        );
        policy.request_open("a", Rect::new(0, 0, 120, 40));
        assert_eq!(policy.collapsed_for("a"), Some(false));
        assert_eq!(policy.collapsed_for("b"), Some(true));
    }

    #[test]
    fn request_open_applies_collapse_rules() {
        let policy = LayoutPolicy::new();
        policy.add_collapse_rule("help", vec!["action".into()]);
        policy.request_open("help", Rect::new(0, 0, 120, 40));
        assert_eq!(policy.collapsed_for("action"), Some(true));
        assert_eq!(policy.collapsed_for("help"), Some(false));
    }

    #[test]
    fn request_open_collapses_on_low_space() {
        let policy = LayoutPolicy::new();
        policy.set_policy("high", SlotPolicy::new(PanelPriority::High));
        policy.set_policy("low", SlotPolicy::new(PanelPriority::Low));
        policy.set_group(
            GroupPolicy::new("compact", vec!["high".into(), "low".into()]).min_area(0, 30),
        );
        policy.request_open("high", Rect::new(0, 0, 120, 10));
        assert_eq!(policy.collapsed_for("high"), Some(false));
        assert_eq!(policy.collapsed_for("low"), Some(true));
    }
}
