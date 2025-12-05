use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleCache {
    pub schedules: HashMap<NaiveDate, Vec<ScheduleBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleBlock {
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub content_id: Option<i32>,
    pub content_path: Option<String>,
    pub script_id: Option<i32>,
}

impl ScheduleCache {
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
        }
    }

    pub fn update(&mut self, date: NaiveDate, blocks: Vec<ScheduleBlock>) {
        self.schedules.insert(date, blocks);
    }

    pub fn get_blocks_for_date(&self, date: NaiveDate) -> Option<&Vec<ScheduleBlock>> {
        self.schedules.get(&date)
    }

    pub fn get_current_block(&self, date: NaiveDate, time: NaiveTime) -> Option<&ScheduleBlock> {
        let blocks = self.get_blocks_for_date(date)?;

        for block in blocks {
            let start = block.start_time;
            let end_secs = start.num_seconds_from_midnight() + (block.duration_minutes as u32 * 60);
            let end = NaiveTime::from_num_seconds_from_midnight_opt(end_secs, 0)?;

            if time >= start && time < end {
                return Some(block);
            }
        }

        None
    }
}
