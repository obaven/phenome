//! Internal policy state and helper routines.

use std::collections::{HashMap, HashSet};

use ratatui::layout::Rect;

use crate::layout::GridSlot;

use super::{GroupPolicy, PanelPriority, SlotOverride, SlotPolicy};
use crate::layout::SlotId;

#[derive(Default)]
pub(super) struct PolicyState {
    pub(super) policies: HashMap<SlotId, SlotPolicy>,
    pub(super) overrides: HashMap<SlotId, SlotOverride>,
    pub(super) groups: Vec<GroupPolicy>,
    pub(super) collapse_rules: HashMap<SlotId, Vec<SlotId>>,
    pub(super) pinned: HashSet<SlotId>,
    pub(super) last_used: HashMap<SlotId, u64>,
    pub(super) usage_counter: u64,
}

impl PolicyState {
    pub(super) fn bump_usage(&mut self, slot: &SlotId) {
        self.usage_counter = self.usage_counter.wrapping_add(1);
        self.last_used.insert(slot.clone(), self.usage_counter);
    }

    pub(super) fn set_collapsed(&mut self, slot: &SlotId, value: bool) {
        let entry = self.overrides.entry(slot.clone()).or_default();
        entry.collapsed = Some(value);
    }

    pub(super) fn apply_collapse_rules(&mut self, slot: &SlotId) {
        if let Some(targets) = self.collapse_rules.get(slot).cloned() {
            for target in targets {
                if self.pinned.contains(&target) {
                    continue;
                }
                self.set_collapsed(&target, true);
            }
        }
    }

    pub(super) fn apply_group_exclusivity(&mut self, slot: &SlotId) {
        let groups = self.groups.clone();
        for group in groups {
            if !group.exclusive || !group.slots.contains(slot) {
                continue;
            }
            for other in &group.slots {
                if other == slot || self.pinned.contains(other) {
                    continue;
                }
                self.set_collapsed(other, true);
            }
        }
    }

    pub(super) fn apply_low_space(&mut self, area: Rect, requested: Option<&SlotId>) {
        let groups = self.groups.clone();
        for group in groups {
            let width_ok = group.min_width == 0 || area.width >= group.min_width;
            let height_ok = group.min_height == 0 || area.height >= group.min_height;
            if width_ok && height_ok {
                continue;
            }
            let mut keep: Vec<SlotId> = group
                .slots
                .iter()
                .filter(|slot| self.pinned.contains(*slot))
                .cloned()
                .collect();
            if let Some(requested) = requested {
                if group.slots.contains(requested) && !keep.contains(requested) {
                    keep.push(requested.clone());
                }
            }
            if keep.is_empty() {
                if let Some(slot) = most_recent_slot(&group, &self.last_used) {
                    keep.push(slot);
                } else if let Some(slot) = highest_priority_slot(&group, &self.policies) {
                    keep.push(slot);
                }
            }
            for slot in &group.slots {
                if keep.contains(slot) {
                    self.set_collapsed(slot, false);
                } else {
                    self.set_collapsed(slot, true);
                }
            }
        }
    }
}

pub(super) fn apply_override(slot: &mut GridSlot, override_data: &SlotOverride) {
    if let Some(visible) = override_data.visible {
        slot.visible = visible;
    }
    if let Some(row) = override_data.row {
        slot.row = row;
    }
    if let Some(col) = override_data.col {
        slot.col = col;
    }
    if let Some(row_span) = override_data.row_span {
        slot.row_span = row_span.max(1);
    }
    if let Some(col_span) = override_data.col_span {
        slot.col_span = col_span.max(1);
    }
}

fn most_recent_slot(group: &GroupPolicy, usage: &HashMap<SlotId, u64>) -> Option<SlotId> {
    group
        .slots
        .iter()
        .filter_map(|slot| usage.get(slot).map(|count| (count, slot)))
        .max_by_key(|(count, _)| *count)
        .map(|(_, slot)| slot.clone())
}

fn highest_priority_slot(
    group: &GroupPolicy,
    policies: &HashMap<SlotId, SlotPolicy>,
) -> Option<SlotId> {
    group
        .slots
        .iter()
        .max_by_key(|slot| {
            policies
                .get(*slot)
                .map(|policy| policy.priority.rank())
                .unwrap_or(PanelPriority::Normal.rank())
        })
        .cloned()
}
