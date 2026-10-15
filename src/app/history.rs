//! Cached grouping and a prefix-sum index for variable-height message rows.

use std::collections::HashMap;

use crate::model::Message;

const DAY_DIVIDER: f32 = 38.0;

#[derive(Clone, Copy, Debug)]
struct Row {
    day: bool,
    grouped: bool,
    height: f32,
}

#[derive(Debug, Default)]
pub struct HistoryLayout {
    rows: Vec<Row>,
    sums: Vec<f64>,
    width: f32,
    font_revision: u64,
    channel: bool,
    first: String,
    last: String,
    valid: bool,
}

impl HistoryLayout {
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub(crate) fn prepare(
        &mut self,
        messages: &[Message],
        width: f32,
        font_revision: u64,
        channel: bool,
        heights: &mut HashMap<String, f32>,
        estimate: impl Fn(&Message, bool) -> f32,
    ) {
        let first = messages.first().map_or("", |m| m.id.as_str());
        let last = messages.last().map_or("", |m| m.id.as_str());
        let changed_width = (self.width - width).abs() > 0.5 || self.font_revision != font_revision;
        if self.valid
            && !changed_width
            && self.channel == channel
            && self.rows.len() == messages.len()
            && self.first == first
            && self.last == last
        {
            return;
        }
        if changed_width {
            heights.clear();
        }
        self.rows.clear();
        let mut previous: Option<&Message> = None;
        let mut previous_day = None;
        for message in messages {
            let day_key = crate::time::day_key(message.created);
            let day = previous_day != Some(day_key);
            let grouped = !channel
                && !day
                && !message.system
                && previous.is_some_and(|p| {
                    !p.system
                        && p.author_id == message.author_id
                        && message.created - p.created < 5 * 60 * 1000
                });
            let height = heights
                .get(&message.id)
                .copied()
                .unwrap_or_else(|| estimate(message, grouped));
            self.rows.push(Row {
                day,
                grouped,
                height: height + if day { DAY_DIVIDER } else { 0.0 },
            });
            previous = Some(message);
            previous_day = Some(day_key);
        }
        self.rebuild_sums();
        self.width = width;
        self.font_revision = font_revision;
        self.channel = channel;
        self.first = first.into();
        self.last = last.into();
        self.valid = true;
    }

    fn rebuild_sums(&mut self) {
        self.sums.clear();
        self.sums.resize(self.rows.len() + 1, 0.0);
        for i in 1..self.sums.len() {
            self.sums[i] += f64::from(self.rows[i - 1].height);
            let parent = i + i.isolate_lowest_one();
            if parent < self.sums.len() {
                self.sums[parent] += self.sums[i];
            }
        }
    }

    pub(crate) fn flags(&self, index: usize) -> (bool, bool) {
        (self.rows[index].day, self.rows[index].grouped)
    }

    pub(crate) fn prefix(&self, mut end: usize) -> f32 {
        let mut sum = 0.0;
        while end > 0 {
            sum += self.sums[end];
            end &= end - 1;
        }
        sum as f32
    }

    pub(crate) fn total(&self) -> f32 {
        self.prefix(self.rows.len())
    }

    fn row_at(&self, offset: f32) -> usize {
        let mut index = 0;
        let mut sum = 0.0;
        let mut step = self.rows.len().next_power_of_two();
        while step > 0 {
            let next = index + step;
            if next < self.sums.len() && sum + self.sums[next] <= f64::from(offset) {
                index = next;
                sum += self.sums[next];
            }
            step >>= 1;
        }
        index.min(self.rows.len())
    }

    pub(crate) fn visible(&self, top: f32, bottom: f32) -> std::ops::Range<usize> {
        self.row_at(top.max(0.0))..(self.row_at(bottom.max(0.0)) + 1).min(self.rows.len())
    }

    pub(crate) fn measure(&mut self, index: usize, height: f32) {
        let row = &mut self.rows[index];
        let height = height + if row.day { DAY_DIVIDER } else { 0.0 };
        let delta = f64::from(height - row.height);
        row.height = height;
        let mut i = index + 1;
        while i < self.sums.len() {
            self.sums[i] += delta;
            i += i.isolate_lowest_one();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_histories_find_the_viewport_and_update_only_changed_prefixes() {
        let mut layout = HistoryLayout {
            rows: vec![
                Row {
                    day: false,
                    grouped: false,
                    height: 50.0
                };
                100_000
            ],
            ..Default::default()
        };
        layout.rebuild_sums();
        assert_eq!(layout.visible(2_000_000.0, 2_001_000.0), 40_000..40_021);
        assert_eq!(layout.total(), 5_000_000.0);
        layout.measure(0, 100.0);
        assert_eq!(layout.prefix(1), 100.0);
        assert_eq!(layout.prefix(2), 150.0);
        assert_eq!(layout.total(), 5_000_050.0);
        assert_eq!(layout.visible(2_000_050.0, 2_001_050.0), 40_000..40_021);
    }
}
