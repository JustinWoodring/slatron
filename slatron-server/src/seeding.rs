use crate::db::DbPool;
use crate::models::{
    Bumper, BumperBack, NewBumper, NewBumperBack, NewGlobalSetting, NewScript, Script,
};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

// Define default settings
const DEFAULT_SETTINGS: &[(&str, &str, &str)] = &[
    ("station_name", "Slatron TV", "The name of the station."),
    (
        "station_bug_image",
        "http://localhost:8080/favicon.png",
        "Path to the station bug image.",
    ),
    (
        "station_theme_color",
        "#0066cc",
        "Primary theme color for station branding (hex code).",
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
    DefaultScript {
        name: "Auto Station Bumpers",
        script_type: "global",
        content: include_str!("defaults/scripts/auto_bumpers.rhai"),
        description: "Automatically plays station idents and transitions based on time.",
        params_schema: None,
    },
];

// Define default bumpers
struct DefaultBumper {
    name: &'static str,
    bumper_type: &'static str,
    template_content: &'static str,
    description: &'static str,
}

// Define default bumper backs
struct DefaultBumperBack {
    name: &'static str,
    mlt_content: &'static str,
    description: &'static str,
}

const DEFAULT_BUMPER_BACKS: &[DefaultBumperBack] = &[
    DefaultBumperBack {
        name: "Solid Blue",
        mlt_content: include_str!("defaults/bumper_backs/solid_blue.mlt"),
        description: "Simple solid blue background.",
    },
    DefaultBumperBack {
        name: "Solid Purple",
        mlt_content: include_str!("defaults/bumper_backs/solid_purple.mlt"),
        description: "Simple solid purple background.",
    },
    DefaultBumperBack {
        name: "Solid Grey",
        mlt_content: include_str!("defaults/bumper_backs/solid_grey.mlt"),
        description: "Simple solid grey background.",
    },
];

const DEFAULT_BUMPERS: &[DefaultBumper] = &[DefaultBumper {
    name: "Station Ident",
    bumper_type: "station_ident",
    template_content: include_str!("defaults/bumpers/station_ident.mlt"),
    description: "5-second station identification with name and theme color.",
}];

pub fn seed_defaults(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get()?;
    tracing::info!("Seeding default values...");

    seed_settings(&mut conn)?;
    seed_scripts(&mut conn)?;
    seed_bumper_backs(&mut conn)?;
    seed_bumpers(&mut conn)?;
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

fn seed_bumper_backs(conn: &mut SqliteConnection) -> Result<()> {
    use crate::schema::bumper_backs::dsl::*;

    for bb in DEFAULT_BUMPER_BACKS {
        // Check if exists by name
        let existing: Option<BumperBack> = bumper_backs
            .filter(name.eq(bb.name))
            .first(conn)
            .optional()?;

        // Store MLT content as "file_path" for now - it will be rendered to MP4 later
        let mlt_path = format!(
            "bumper_backs/{}.mlt",
            bb.name.to_lowercase().replace(" ", "_")
        );

        if let Some(back) = existing {
            // Update if built-in
            if back.is_builtin {
                tracing::info!("Updating built-in bumper back: {}", bb.name);
                diesel::update(bumper_backs.filter(id.eq(back.id)))
                    .set((
                        file_path.eq(&mlt_path),
                        description.eq(Some(bb.description)),
                    ))
                    .execute(conn)?;
            }
        } else {
            // Insert new
            tracing::info!("Seeding bumper back: {}", bb.name);

            // Write MLT file to disk
            let mlt_file_path = std::path::PathBuf::from("static/media").join(&mlt_path);
            if let Some(parent) = mlt_file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&mlt_file_path, bb.mlt_content)?;

            let new_back = NewBumperBack {
                name: bb.name.to_string(),
                description: Some(bb.description.to_string()),
                file_path: format!("media/{}", mlt_path),
                duration_ms: None, // Will be set when rendered
                is_builtin: true,
            };

            diesel::insert_into(bumper_backs)
                .values(&new_back)
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

fn seed_bumpers(conn: &mut SqliteConnection) -> Result<()> {
    use crate::schema::bumper_backs::dsl as bb_dsl;
    use crate::schema::bumpers::dsl::*;

    // Get the default bumper back ID (Solid Blue)
    let default_back_id: Option<i32> = bb_dsl::bumper_backs
        .filter(bb_dsl::name.eq("Solid Blue"))
        .select(bb_dsl::id)
        .first(conn)
        .optional()?
        .flatten();

    for db in DEFAULT_BUMPERS {
        // Check if exists by name
        let existing: Option<Bumper> = bumpers.filter(name.eq(db.name)).first(conn).optional()?;

        if let Some(bumper) = existing {
            // Update template content if it's built-in (ensure built-in bumpers stay up to date)
            if bumper.is_builtin {
                tracing::info!("Updating built-in bumper: {}", db.name);
                diesel::update(bumpers.filter(id.eq(bumper.id)))
                    .set((
                        template_content.eq(db.template_content),
                        description.eq(Some(db.description)),
                        bumper_type.eq(db.bumper_type),
                        bumper_back_id.eq(default_back_id),
                    ))
                    .execute(conn)?;
            }
        } else {
            // Insert new
            tracing::info!("Seeding bumper: {}", db.name);
            let new_bumper = NewBumper {
                name: db.name.to_string(),
                bumper_type: db.bumper_type.to_string(),
                description: Some(db.description.to_string()),
                is_template: true,
                template_content: Some(db.template_content.to_string()),
                rendered_path: None,
                duration_ms: None,
                is_builtin: true,
                bumper_back_id: default_back_id,
            };

            diesel::insert_into(bumpers)
                .values(&new_bumper)
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
