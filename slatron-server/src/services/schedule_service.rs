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

    // Map Cache for DJ Names to avoid N+1
    let mut dj_names_cache: HashMap<i32, String> = HashMap::new();

    // Fetch relevant DJs
    // First, collect all DJ IDs from all schedules in cache
    let mut all_dj_ids = Vec::new();
    for (_, blocks) in schedule_blocks_cache.iter() {
        for b in blocks {
            if let Some(did) = b.dj_id {
                all_dj_ids.push(did);
            }
        }
    }
    all_dj_ids.sort();
    all_dj_ids.dedup();

    if !all_dj_ids.is_empty() {
        use crate::schema::dj_profiles::dsl::*;
        let fetched_djs = dj_profiles
            .filter(id.eq_any(all_dj_ids))
            .select((crate::schema::dj_profiles::dsl::id, name))
            .load::<(Option<i32>, String)>(conn)?; // Load as Option<i32> because column is nullable
        for (did_opt, dname) in fetched_djs {
            if let Some(did) = did_opt {
                dj_names_cache.insert(did, dname);
            }
        }
    }

    // 4. Fill a 1440-minute LOCAL-day timeline: for each minute the
    //    highest-priority schedule with a covering block wins. Blocks from
    //    `date - 1` that cross midnight cover their spill-over tail.
    let fill_schedules: Vec<FillSchedule> = effective_schedules
        .iter()
        .map(|item| FillSchedule {
            id: item.schedule.id.unwrap(),
            name: item.schedule.name.clone(),
            effective_priority: item.effective_priority,
        })
        .collect();

    let timeline = fill_timeline(&fill_schedules, &schedule_blocks_cache, date, &dj_names_cache);

    // 5. Collapse adjacent identical blocks
    let collapsed = collapse_timeline(timeline);

    Ok(collapsed)
}

/// A schedule reduced to what the timeline fill needs, pre-sorted highest
/// priority first by the caller.
pub(crate) struct FillSchedule {
    pub id: i32,
    pub name: String,
    pub effective_priority: i32,
}

fn covering_block<'a>(blocks: &'a [ScheduleBlock], secs: u32) -> Option<&'a ScheduleBlock> {
    blocks.iter().find(|b| {
        let start =
            b.start_time.hour() * 3600 + b.start_time.minute() * 60 + b.start_time.second();
        let end = start + b.duration_minutes as u32 * 60;
        secs >= start && secs < end
    })
}

/// Fills the 1440-minute timeline for `date`.
///
/// Blocks are looked up for `date` itself, and for `date - 1` when the block
/// ends after midnight: minute `m` of `date` is covered by a yesterday block
/// whose interval contains `m + 1440` minutes. This is what makes overnight
/// blocks (e.g. 23:30 + 90min) visible on both sides of midnight.
pub(crate) fn fill_timeline(
    schedules: &[FillSchedule],
    blocks_by_schedule_date: &HashMap<(i32, NaiveDate), Vec<ScheduleBlock>>,
    date: NaiveDate,
    dj_names: &HashMap<i32, String>,
) -> Vec<Option<TimelineSlot>> {
    let mut timeline: Vec<Option<TimelineSlot>> = vec![None; 1440];
    let yesterday = date.pred_opt();

    for (minute, slot) in timeline.iter_mut().enumerate() {
        let local_secs = (minute as u32) * 60;
        'schedules: for sched in schedules {
            if let Some(blocks) = blocks_by_schedule_date.get(&(sched.id, date)) {
                if let Some(block) = covering_block(blocks, local_secs) {
                    *slot = Some(slot_for(sched, block, dj_names));
                    break 'schedules;
                }
            }
            if let Some(ydate) = yesterday {
                if let Some(blocks) = blocks_by_schedule_date.get(&(sched.id, ydate)) {
                    if let Some(block) = covering_block(blocks, local_secs + 86_400) {
                        *slot = Some(slot_for(sched, block, dj_names));
                        break 'schedules;
                    }
                }
            }
        }
    }

    timeline
}

fn slot_for(
    sched: &FillSchedule,
    block: &ScheduleBlock,
    dj_names: &HashMap<i32, String>,
) -> TimelineSlot {
    TimelineSlot {
        content_id: block.content_id,
        script_id: block.script_id,
        priority: sched.effective_priority,
        schedule_name: sched.name.clone(),
        schedule_id: sched.id,
        block_id: block.id.expect("Block ID missing"),
        dj_id: block.dj_id,
        dj_name: block.dj_id.and_then(|did| dj_names.get(&did).cloned()),
    }
}

#[derive(Clone, Debug)]
struct TimelineSlot {
    content_id: Option<i32>,
    script_id: Option<i32>,
    priority: i32,
    schedule_name: String,
    schedule_id: i32,
    block_id: i32,
    dj_id: Option<i32>,
    dj_name: Option<String>,
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
        dj_id: slot.dj_id,
        dj_name: slot.dj_name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use std::collections::HashMap;

    fn time(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn block(id: i32, start: NaiveTime, duration_minutes: i32) -> ScheduleBlock {
        ScheduleBlock {
            id: Some(id),
            schedule_id: 1,
            content_id: Some(100),
            day_of_week: None,
            specific_date: None,
            start_time: start,
            duration_minutes,
            script_id: None,
            dj_id: None,
            created_at: chrono::naive::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: chrono::naive::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    fn sched(id: i32, name: &str, priority: i32) -> FillSchedule {
        FillSchedule {
            id,
            name: name.to_string(),
            effective_priority: priority,
        }
    }

    fn collapse(
        schedules: &[FillSchedule],
        entries: &[(i32, NaiveDate, Vec<ScheduleBlock>)],
        date: NaiveDate,
    ) -> Vec<CollapsedBlock> {
        let mut map: HashMap<(i32, NaiveDate), Vec<ScheduleBlock>> = HashMap::new();
        for (id, d, blocks) in entries {
            map.insert((*id, *d), blocks.clone());
        }
        collapse_timeline(fill_timeline(schedules, &map, date, &HashMap::new()))
    }

    #[test]
    fn overnight_block_spills_across_midnight() {
        // Yesterday 23:30 + 90min → the 00:00–01:00 tail must appear today.
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let yesterday = date.pred_opt().unwrap();
        let collapsed = collapse(
            &[sched(1, "Overnight", 1)],
            &[(
                1,
                yesterday,
                vec![block(10, time(23, 30), 90)],
            )],
            date,
        );

        assert_eq!(collapsed.len(), 1);
        assert_eq!(collapsed[0].start_time, "00:00:00");
        assert_eq!(collapsed[0].duration_minutes, 60);
        assert_eq!(collapsed[0].schedule_name, "Overnight");
    }

    #[test]
    fn overnight_spill_yields_to_higher_priority_today() {
        // Yesterday's 23:30+90 (prio 1) vs today's 00:15+30 (prio 10):
        // today's block wins 00:15–00:45; the spill fills around it.
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let yesterday = date.pred_opt().unwrap();
        let collapsed = collapse(
            &[sched(2, "Prime", 10), sched(1, "Overnight", 1)],
            &[
                (1, yesterday, vec![block(10, time(23, 30), 90)]),
                (2, date, vec![block(20, time(0, 15), 30)]),
            ],
            date,
        );

        assert_eq!(collapsed.len(), 3);
        assert_eq!(
            (collapsed[0].start_time.as_str(), collapsed[0].duration_minutes),
            ("00:00:00", 15)
        );
        assert_eq!(collapsed[1].schedule_name, "Prime");
        assert_eq!(collapsed[1].duration_minutes, 30);
        assert_eq!(
            (collapsed[2].start_time.as_str(), collapsed[2].duration_minutes),
            ("00:45:00", 15)
        );
        assert_eq!(collapsed[2].schedule_name, "Overnight");
    }

    #[test]
    fn priority_tie_prefers_first_schedule() {
        // Equal priorities: the first schedule in priority order wins.
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let collapsed = collapse(
            &[sched(1, "First", 5), sched(2, "Second", 5)],
            &[
                (1, date, vec![block(10, time(10, 0), 30)]),
                (2, date, vec![block(20, time(10, 0), 30)]),
            ],
            date,
        );

        assert_eq!(collapsed.len(), 1);
        assert_eq!(collapsed[0].schedule_name, "First");
    }

    #[test]
    fn lower_priority_never_overrides_covered_minute() {
        // High priority 09:00–10:00; low priority 08:00–11:00.
        // High wins 09:00–10:00; low fills the rest.
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let collapsed = collapse(
            &[sched(2, "High", 10), sched(1, "Low", 1)],
            &[
                (2, date, vec![block(20, time(9, 0), 60)]),
                (1, date, vec![block(10, time(8, 0), 180)]),
            ],
            date,
        );

        assert_eq!(collapsed.len(), 3);
        assert_eq!(
            (collapsed[0].start_time.as_str(), collapsed[0].duration_minutes),
            ("08:00:00", 60)
        );
        assert_eq!(collapsed[1].schedule_name, "High");
        assert_eq!(collapsed[1].duration_minutes, 60);
        assert_eq!(
            (collapsed[2].start_time.as_str(), collapsed[2].duration_minutes),
            ("10:00:00", 60)
        );
        assert_eq!(collapsed[2].schedule_name, "Low");
    }

    #[test]
    fn no_blocks_yields_empty_schedule() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let collapsed = collapse(&[sched(1, "Empty", 1)], &[], date);
        assert!(collapsed.is_empty());
    }

    #[test]
    fn non_overnight_yesterday_blocks_do_not_leak() {
        // Yesterday 22:00+30 (ends before midnight) must NOT appear today.
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let yesterday = date.pred_opt().unwrap();
        let collapsed = collapse(
            &[sched(1, "Evening", 1)],
            &[(
                1,
                yesterday,
                vec![block(10, time(22, 0), 30)],
            )],
            date,
        );
        assert!(collapsed.is_empty());
    }

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
                dj_id: None,
                dj_name: None,
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
                    dj_id: None,
                    dj_name: None,
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
