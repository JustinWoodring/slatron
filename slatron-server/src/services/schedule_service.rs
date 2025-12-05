use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use diesel::prelude::*;
use crate::api::schedules_api::CollapsedBlock;
use crate::db::DbConnection;
use crate::models::{Schedule, ScheduleBlock};

pub fn calculate_collapsed_schedule(
    conn: &mut DbConnection,
    node_id: i32,
    date: NaiveDate,
) -> Result<Vec<CollapsedBlock>> {
    use crate::schema::{node_schedules, schedules, schedule_blocks};

    // Get all schedules assigned to this node
    let assigned_schedules: Vec<Schedule> = node_schedules::table
        .inner_join(schedules::table)
        .filter(node_schedules::node_id.eq(node_id))
        .filter(schedules::is_active.eq(true))
        .select(Schedule::as_select())
        .load(conn)?;

    // Sort by priority (descending - higher priority first)
    let mut sorted_schedules = assigned_schedules;
    sorted_schedules.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Create a 1440-minute timeline (24 hours * 60 minutes)
    let mut timeline: Vec<Option<TimelineSlot>> = vec![None; 1440];

    // Process each schedule in priority order
    for schedule in sorted_schedules.iter() {
        let blocks = get_blocks_for_date(conn, schedule, date)?;

        for block in blocks {
            // Convert start time to minutes since midnight
            let start_min = (block.start_time.hour() as usize * 60)
                + block.start_time.minute() as usize;
            let end_min = start_min + block.duration_minutes as usize;

            // Fill timeline slots
            for minute in start_min..end_min.min(1440) {
                // Only fill if empty or lower priority
                if timeline[minute].is_none()
                    || timeline[minute].as_ref().unwrap().priority < schedule.priority
                {
                    timeline[minute] = Some(TimelineSlot {
                        content_id: block.content_id,
                        script_id: block.script_id,
                        priority: schedule.priority,
                        schedule_name: schedule.name.clone(),
                        block_id: block.id,
                    });
                }
            }
        }
    }

    // Collapse adjacent identical blocks
    let collapsed = collapse_timeline(timeline);

    Ok(collapsed)
}

#[derive(Clone, Debug)]
struct TimelineSlot {
    content_id: Option<i32>,
    script_id: Option<i32>,
    priority: i32,
    schedule_name: String,
    block_id: i32,
}

impl PartialEq for TimelineSlot {
    fn eq(&self, other: &Self) -> bool {
        self.content_id == other.content_id
            && self.script_id == other.script_id
            && self.block_id == other.block_id
    }
}

fn get_blocks_for_date(
    conn: &mut DbConnection,
    schedule: &Schedule,
    date: NaiveDate,
) -> Result<Vec<ScheduleBlock>> {
    use crate::schema::schedule_blocks::dsl;

    let blocks = match schedule.schedule_type.as_str() {
        "weekly" => {
            // Sunday = 0, Monday = 1, etc.
            let day_of_week = date.weekday().num_days_from_sunday() as i32;

            dsl::schedule_blocks
                .filter(dsl::schedule_id.eq(schedule.id))
                .filter(dsl::day_of_week.eq(day_of_week))
                .select(ScheduleBlock::as_select())
                .load(conn)?
        }
        "one_off" => {
            dsl::schedule_blocks
                .filter(dsl::schedule_id.eq(schedule.id))
                .filter(dsl::specific_date.eq(date))
                .select(ScheduleBlock::as_select())
                .load(conn)?
        }
        _ => vec![],
    };

    Ok(blocks)
}

fn collapse_timeline(timeline: Vec<Option<TimelineSlot>>) -> Vec<CollapsedBlock> {
    let mut collapsed = Vec::new();
    let mut current_block: Option<(usize, TimelineSlot)> = None;

    for (minute, slot_opt) in timeline.iter().enumerate() {
        match (slot_opt, &current_block) {
            (Some(slot), Some((start_min, current_slot))) => {
                // Check if this continues the current block
                if slot == current_slot {
                    // Continue the current block
                    continue;
                } else {
                    // Different block, save the current one
                    let duration = minute - start_min;
                    collapsed.push(create_collapsed_block(
                        start_min,
                        duration as i32,
                        current_slot,
                    ));

                    // Start a new block
                    current_block = Some((minute, slot.clone()));
                }
            }
            (Some(slot), None) => {
                // Start a new block
                current_block = Some((minute, slot.clone()));
            }
            (None, Some((start_min, current_slot))) => {
                // End of current block
                let duration = minute - start_min;
                collapsed.push(create_collapsed_block(
                    start_min,
                    duration as i32,
                    current_slot,
                ));
                current_block = None;
            }
            (None, None) => {
                // No block continues
            }
        }
    }

    // Handle any remaining block at end of day
    if let Some((start_min, current_slot)) = current_block {
        let duration = 1440 - start_min;
        collapsed.push(create_collapsed_block(
            start_min,
            duration as i32,
            &current_slot,
        ));
    }

    collapsed
}

fn create_collapsed_block(
    start_min: usize,
    duration: i32,
    slot: &TimelineSlot,
) -> CollapsedBlock {
    let hours = start_min / 60;
    let minutes = start_min % 60;
    let start_time = format!("{:02}:{:02}:00", hours, minutes);

    CollapsedBlock {
        start_time,
        duration_minutes: duration,
        content_id: slot.content_id,
        script_id: slot.script_id,
        priority: slot.priority,
        schedule_name: slot.schedule_name.clone(),
    }
}
