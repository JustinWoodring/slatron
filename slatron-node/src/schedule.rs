use chrono::{NaiveDate, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

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
        debug!("Checking schedule for Date: {:?}, Time: {:?}", date, time);

        let blocks = match self.get_blocks_for_date(date) {
            Some(b) => b,
            None => {
                debug!("No schedule blocks found for date {:?}", date);
                debug!("Available dates in cache: {:?}", self.schedules.keys());
                return self.get_spillover_block(date, time);
            }
        };

        if let Some(matched) = self.find_covering_block(blocks, time) {
            return Some(matched);
        }

        self.get_spillover_block(date, time)
    }

    fn find_covering_block<'a>(
        &self,
        blocks: &'a [ScheduleBlock],
        time: NaiveTime,
    ) -> Option<&'a ScheduleBlock> {
        let current_secs = time.hour() * 3600 + time.minute() * 60 + time.second();

        for block in blocks {
            let start = block.start_time;
            let start_secs = start.hour() * 3600 + start.minute() * 60 + start.second();
            let end_secs = start_secs + (block.duration_minutes as u32 * 60);

            if current_secs >= start_secs && current_secs < end_secs {
                info!("  -> MATCH! Playing block content {:?}", block.content_id);
                return Some(block);
            }
        }

        None
    }

    /// Overnight blocks from the previous day (e.g. 23:30 + 90min) cover the
    /// first minutes of `date`: probe the previous day's cache with
    /// `time + 24h` so the tail of a block that crosses midnight still plays.
    fn get_spillover_block(&self, date: NaiveDate, time: NaiveTime) -> Option<&ScheduleBlock> {
        let yesterday = date.pred_opt()?;
        let yesterday_blocks = self.get_blocks_for_date(yesterday)?;

        let current_secs = time.hour() * 3600 + time.minute() * 60 + time.second() + 86_400;

        for block in yesterday_blocks {
            let start_secs =
                block.start_time.hour() * 3600 + block.start_time.minute() * 60 + block.start_time.second();
            let end_secs = start_secs + (block.duration_minutes as u32 * 60);

            if end_secs > 86_400 && current_secs >= start_secs && current_secs < end_secs {
                info!(
                    "  -> MATCH (overnight from {:?})! Playing block content {:?}",
                    yesterday, block.content_id
                );
                return Some(block);
            }
        }

        debug!("  -> No matching block found for time {:?}", time);
        None
    }
}
