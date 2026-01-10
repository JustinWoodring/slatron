use crate::db::DbPool;
use crate::models::{NewGlobalSetting, NewScript, Script};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

// Define default settings
const DEFAULT_SETTINGS: &[(&str, &str, &str)] = &[
    (
        "content_error_behavior",
        "skip",
        "skip | retry | error_screen | previous",
    ),
    (
        "content_error_retry_attempts",
        "3",
        "Retry attempts on content error",
    ),
    (
        "content_error_retry_delay_secs",
        "5",
        "Delay between retries",
    ),
    (
        "node_heartbeat_timeout_secs",
        "15",
        "Mark node offline after N seconds",
    ),
    ("default_transition_type", "cut", "cut | fade"),
    ("station_name", "Slatron TV", "The name of the station."),
    (
        "station_bug_image",
        "http://localhost:8080/favicon.png",
        "Path to the station bug image.",
    ),
    (
        "global_active_scripts",
        "[\"Station Bug\"]",
        "JSON array of Script Names to execute for every content item.",
    ),
    (
        "onboarding_complete",
        "false",
        "Whether the initial onboarding wizard has been completed.",
    ),
    ("timezone", "UTC", "Global timezone for the station"),
];

// Define default scripts
struct DefaultScript {
    name: &'static str,
    script_type: &'static str,
    content: &'static str,
    description: &'static str,
    params_schema: Option<&'static str>,
}

const DEFAULT_SCRIPTS: &[DefaultScript] = &[
    DefaultScript {
        name: "Loop Content",
        script_type: "transformer",
        content: include_str!("defaults/scripts/loop_content.rhai"),
        description: "Loops the content item.",
        params_schema: None,
    },
    DefaultScript {
        name: "Mute Audio",
        script_type: "transformer",
        content: include_str!("defaults/scripts/mute_audio.rhai"),
        description: "Mutes audio playback.",
        params_schema: None,
    },
    DefaultScript {
        name: "Start at 10s",
        script_type: "transformer",
        content: include_str!("defaults/scripts/start_at_10s.rhai"),
        description: "Starts playback at the 10-second mark.",
        params_schema: None,
    },
    DefaultScript {
        name: "Station Bug",
        script_type: "transformer",
        content: include_str!("defaults/scripts/station_bug.rhai"),
        description: "Displays the station bug based on global settings.",
        params_schema: None,
    },
    DefaultScript {
        name: "YouTube Loader",
        script_type: "content_loader",
        content: include_str!("defaults/scripts/youtube_loader.rhai"),
        description: "Loads content from a YouTube URL (Video or Playlist).",
        params_schema: Some(include_str!("defaults/scripts/youtube_loader.params.json")),
    },
    DefaultScript {
        name: "Schedule Context",
        script_type: "server_context",
        content: include_str!("defaults/scripts/schedule_context.rhai"),
        description: "Injects current block info and upcoming schedule items.",
        params_schema: None,
    },
    DefaultScript {
        name: "Time Injector",
        script_type: "server_context",
        content: include_str!("defaults/scripts/time_injector.rhai"),
        description: "Injects current time into DJ context.",
        params_schema: None,
    },
    DefaultScript {
        name: "RSS News Feed",
        script_type: "server_context",
        content: include_str!("defaults/scripts/rss_news.rhai"),
        description: "Fetches RSS/Atom feed and injects it into local news context.",
        params_schema: Some(include_str!("defaults/scripts/rss_news.params.json")),
    },
];

pub fn seed_defaults(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get()?;
    tracing::info!("Seeding default values...");

    seed_settings(&mut conn)?;
    seed_scripts(&mut conn)?;
    seed_users(&mut conn)?;

    Ok(())
}

fn seed_settings(conn: &mut SqliteConnection) -> Result<()> {
    use crate::schema::global_settings::dsl::*;

    for (key_val, val_val, desc) in DEFAULT_SETTINGS {
        // Check if exists
        let exists: i64 = global_settings
            .filter(key.eq(key_val))
            .count()
            .get_result(conn)?;

        if exists == 0 {
            tracing::info!("Seeding setting: {}", key_val);
            let new_setting = NewGlobalSetting {
                key: key_val.to_string(),
                value: val_val.to_string(),
                description: Some(desc.to_string()),
            };

            diesel::insert_into(global_settings)
                .values(&new_setting)
                .execute(conn)?;
        }
    }
    Ok(())
}

fn seed_scripts(conn: &mut SqliteConnection) -> Result<()> {
    use crate::schema::scripts::dsl::*;

    for ds in DEFAULT_SCRIPTS {
        // Check if exists by name
        let existing: Option<Script> = scripts.filter(name.eq(ds.name)).first(conn).optional()?;

        if let Some(script) = existing {
            // Update content if it's built-in (ensure built-in scripts stay up to date)
            if script.is_builtin {
                tracing::info!("Updating built-in script: {}", ds.name);
                diesel::update(scripts.filter(id.eq(script.id)))
                    .set((
                        script_content.eq(ds.content),
                        description.eq(ds.description),
                        script_type.eq(ds.script_type),
                        parameters_schema.eq(ds.params_schema),
                    ))
                    .execute(conn)?;
            }
        } else {
            // Insert new
            tracing::info!("Seeding script: {}", ds.name);
            let new_script = NewScript {
                name: ds.name.to_string(),
                description: Some(ds.description.to_string()),
                script_type: ds.script_type.to_string(),
                script_content: ds.content.to_string(),
                parameters_schema: ds.params_schema.map(|s| s.to_string()),
                is_builtin: true,
            };

            diesel::insert_into(scripts)
                .values(&new_script)
                .execute(conn)?;
        }
    }
    Ok(())
}

fn seed_users(conn: &mut SqliteConnection) -> Result<()> {
    use crate::schema::users::dsl::*;
    // Check if admin user exists
    let exists: i64 = users
        .filter(username.eq("admin"))
        .count()
        .get_result(conn)?;

    if exists == 0 {
        tracing::info!("Seeding user: admin");
        // Hash password "admin"
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash_str = argon2
            .hash_password(b"admin", &salt)
            .map_err(|e| anyhow::anyhow!("Hashing failed: {}", e))?
            .to_string();

        let new_user = crate::models::NewUser {
            username: "admin".to_string(),
            password_hash: password_hash_str,
            role: "admin".to_string(),
        };

        diesel::insert_into(users).values(&new_user).execute(conn)?;
    }

    Ok(())
}
