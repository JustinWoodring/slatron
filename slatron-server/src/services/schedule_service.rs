use crate::api::schedules_api::CollapsedBlock;
use crate::db::DbConnection;
use crate::models::{Schedule, ScheduleBlock};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Timelike};
use diesel::prelude::*;
use std::collections::HashMap;

pub fn calculate_collapsed_schedule(
    conn: &mut DbConnection,
    node_id: i32,
    date: NaiveDate,
    _timezone_str: Option<String>,
) -> Result<Vec<CollapsedBlock>> {
    use crate::schema::{node_schedules, schedules};

    // 1. Get all schedules assigned to this node along with assignment priority
    let assigned_data: Vec<(Schedule, Option<i32>)> = node_schedules::table
        .inner_join(schedules::table)
        .filter(node_schedules::node_id.eq(node_id))
        .filter(schedules::is_active.eq(true))
        .select((Schedule::as_select(), node_schedules::priority))
        .load(conn)?;

    struct EffectiveSchedule {
        schedule: Schedule,
        effective_priority: i32,
    }

    let mut effective_schedules: Vec<EffectiveSchedule> = assigned_data
        .into_iter()
        .map(|(s, p_override)| {
            let eff = p_override.unwrap_or(s.priority);
            EffectiveSchedule {
                schedule: s,
                effective_priority: eff,
            }
        })
        .collect();

    // Sort by priority (descending - higher priority first)
    effective_schedules.sort_by(|a, b| b.effective_priority.cmp(&a.effective_priority));

    // 2. Parse Timezone (Not strictly used here anymore as we rely on 'date' being localized,
    // but good to validate string if needed, or remove)
    // let tz: chrono_tz::Tz = timezone_str ... (Removed as unused)

    // 3. Pre-fetch blocks for relevant dates (Yesterday, Today, Tomorrow)
    //    because local time might shift across midnight relative to UTC.
    //    We cheat slightly and just fetch blocks for the target date and adjacent days.
    //    Actually, simpler: Just fetch blocks for `date`, `date - 1`, `date + 1`.
    let valid_dates = [date.pred_opt().unwrap(), date, date.succ_opt().unwrap()];

    // Map: (ScheduleID, Date) -> Vec<ScheduleBlock>
    let mut schedule_blocks_cache: HashMap<(i32, NaiveDate), Vec<ScheduleBlock>> = HashMap::new();

    for item in &effective_schedules {
        for d in valid_dates {
            let blocks = get_blocks_for_date(conn, &item.schedule, d)?;
            schedule_blocks_cache.insert((item.schedule.id.unwrap(), d), blocks);
        }
    }

    // 4. Create a 1440-minute timeline (24 hours * 60 minutes) representing LOCAL DAY
    let mut timeline: Vec<Option<TimelineSlot>> = vec![None; 1440];

    // 5. Fill Timeline
    //    Iterate 0..1440 (LOCAL minutes).
    //    Find matching block in Highest Priority Schedule.
    for local_minute in 0..1440 {
        // Find highest priority schedule that has a block at this local time
        for item in &effective_schedules {
            let schedule_id = item.schedule.id.unwrap();

            // We use the requested 'date' as the LOCAL date.
            if let Some(blocks) = schedule_blocks_cache.get(&(schedule_id, date)) {
                // Check if any block covers this local_time
                let mut match_found = false;
                for block in blocks {
                    let start = block.start_time;
                    let end_secs = start.hour() * 3600
                        + start.minute() * 60
                        + start.second()
                        + (block.duration_minutes as u32 * 60);
                    // let end = chrono::NaiveTime::from_num_seconds_from_midnight_opt(end_secs, 0);

                    let local_secs = local_minute * 60;
                    let start_secs_val = start.hour() * 3600 + start.minute() * 60 + start.second();

                    if local_secs >= start_secs_val && local_secs < end_secs {
                        // Found a match!
                        timeline[local_minute as usize] = Some(TimelineSlot {
                            content_id: block.content_id,
                            script_id: block.script_id,
                            priority: item.effective_priority,
                            schedule_name: item.schedule.name.clone(),
                            schedule_id: schedule_id,
                            block_id: block.id.expect("Block ID missing"),
                        });
                        match_found = true;
                        break;
                    }
                }
                if match_found {
                    break; // Stop checking lower priority schedules for this minute
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
    schedule_id: i32,
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
            // Monday = 0, Tuesday = 1, etc. (To match Frontend array indices)
            let day_of_week = date.weekday().num_days_from_monday() as i32;

            dsl::schedule_blocks
                .filter(dsl::schedule_id.eq(schedule.id.expect("Schedule ID missing")))
                .filter(dsl::day_of_week.eq(day_of_week))
                .select(ScheduleBlock::as_select())
                .load(conn)?
        }
        "one_off" => dsl::schedule_blocks
            .filter(dsl::schedule_id.eq(schedule.id.expect("Schedule ID missing")))
            .filter(dsl::specific_date.eq(date))
            .select(ScheduleBlock::as_select())
            .load(conn)?,
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
                    let duration = minute - *start_min;
                    collapsed.push(create_collapsed_block(
                        *start_min,
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
                let duration = minute - *start_min;
                collapsed.push(create_collapsed_block(
                    *start_min,
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

fn create_collapsed_block(start_min: usize, duration: i32, slot: &TimelineSlot) -> CollapsedBlock {
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
        schedule_id: slot.schedule_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_clipping_logic() {
        // Simulate the painting logic
        let mut timeline: Vec<Option<TimelineSlot>> = vec![None; 1440];

        // 1. High Priority (Priority 10): 1:30 - 2:00 (90m - 120m)
        let hp_start = 90;
        let hp_end = 120;
        let hp_priority = 10;

        for minute in hp_start..hp_end {
            timeline[minute] = Some(TimelineSlot {
                content_id: Some(2),
                script_id: None,
                priority: hp_priority,
                schedule_name: "High Pri".to_string(),
                schedule_id: 2,
                block_id: 200,
            });
        }

        // 2. Low Priority (Priority 1): 1:00 - 3:00 (60m - 180m)
        let lp_start = 60;
        let lp_end = 180;
        let lp_priority = 1;

        // Apply Low Priority (Gap Filling)
        for minute in lp_start..lp_end {
            // Logic: Fill if empty or existing is lower priority (which shouldn't happen if sorted High->Low)
            // But here we simulate the High->Low iteration order.
            // Since HP is already there with prio 10, and we have prio 1, we only fill if None.
            if timeline[minute].is_none() {
                timeline[minute] = Some(TimelineSlot {
                    content_id: Some(1),
                    script_id: None,
                    priority: lp_priority,
                    schedule_name: "Low Pri".to_string(),
                    schedule_id: 1,
                    block_id: 100,
                });
            }
        }

        // 3. Verify Timeline
        // 60-90: Should be Low Pri
        for i in 60..90 {
            assert_eq!(
                timeline[i].as_ref().unwrap().priority,
                1,
                "Minute {} should be Low Pri",
                i
            );
        }
        // 90-120: Should be High Pri
        for i in 90..120 {
            assert_eq!(
                timeline[i].as_ref().unwrap().priority,
                10,
                "Minute {} should be High Pri",
                i
            );
        }
        // 120-180: Should be Low Pri
        for i in 120..180 {
            assert_eq!(
                timeline[i].as_ref().unwrap().priority,
                1,
                "Minute {} should be Low Pri",
                i
            );
        }

        // 4. Verify Collapse
        let collapsed = collapse_timeline(timeline);

        // Expecting 3 blocks: Low (30m), High (30m), Low (60m)
        assert_eq!(collapsed.len(), 3);

        assert_eq!(collapsed[0].schedule_name, "Low Pri");
        assert_eq!(collapsed[0].duration_minutes, 30);

        assert_eq!(collapsed[1].schedule_name, "High Pri");
        assert_eq!(collapsed[1].duration_minutes, 30);

        assert_eq!(collapsed[2].schedule_name, "Low Pri");
        assert_eq!(collapsed[2].duration_minutes, 60);
    }
}
