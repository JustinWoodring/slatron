use crate::db::DbPool;
use crate::models::{Bumper, BumperBack};
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub struct BumperService {
    db: DbPool,
}

impl BumperService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Substitute template variables in MLT XML
    fn substitute_variables(
        &self,
        template: &str,
        station_name: &str,
        theme_color: &str,
        bumper_back_path: Option<&str>,
    ) -> String {
        let mut result = template
            .replace("{{STATION_NAME}}", &Self::escape_xml(station_name))
            .replace("{{THEME_COLOR}}", &Self::escape_xml(theme_color));

        if let Some(path) = bumper_back_path {
            // Path shouldn't strictly need XML escaping if it's a valid path,
            // but for safety in XML attribute:
            result = result.replace("{{BUMPER_BACK_PATH}}", &Self::escape_xml(path));
        }

        result
    }

    /// Extract video duration in milliseconds using ffprobe
    fn get_duration_ms(&self, video_path: &PathBuf) -> Result<i32> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(video_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ffprobe failed: {}", stderr));
        }

        let duration_str = String::from_utf8(output.stdout)?;
        let duration_secs: f32 = duration_str.trim().parse()?;
        let duration_ms = (duration_secs * 1000.0) as i32;

        Ok(duration_ms)
    }

    /// Get station settings needed for template substitution
    fn get_station_settings(&self) -> Result<(String, String)> {
        use crate::schema::global_settings::dsl::*;
        let mut conn = self.db.get()?;

        let station_name_value: String = global_settings
            .filter(key.eq("station_name"))
            .select(value)
            .first(&mut conn)
            .optional()?
            .unwrap_or_else(|| "Slatron TV".to_string());

        let theme_color_value: String = global_settings
            .filter(key.eq("station_theme_color"))
            .select(value)
            .first(&mut conn)
            .optional()?
            .unwrap_or_else(|| "#0066cc".to_string());

        Ok((station_name_value, theme_color_value))
    }

    /// Render a single bumper template to MP4
    pub fn render_template(&self, bumper_id: i32) -> Result<PathBuf> {
        use crate::schema::bumper_backs::dsl as bb_dsl;
        use crate::schema::bumpers::dsl::*;
        let mut conn = self.db.get()?;

        // Get bumper from database
        let bumper: Bumper = bumpers.filter(id.eq(Some(bumper_id))).first(&mut conn)?;

        if !bumper.is_template {
            return Err(anyhow!("Bumper {} is not a template", bumper.name));
        }

        let template_str = bumper
            .template_content
            .ok_or_else(|| anyhow!("Template content is empty"))?;

        // Get bumper back if specified
        let bumper_back_path = if let Some(back_id) = bumper.bumper_back_id {
            let back: BumperBack = bb_dsl::bumper_backs
                .filter(bb_dsl::id.eq(Some(back_id)))
                .first(&mut conn)?;
            // Prepend static/ to the path so MLT can find it relative to CWD
            Some(format!("static/{}", back.file_path))
        } else {
            None
        };

        // Get station settings
        let (station_name_value, theme_color_value) = self.get_station_settings()?;

        // Substitute variables
        let substituted = self.substitute_variables(
            &template_str,
            &station_name_value,
            &theme_color_value,
            bumper_back_path.as_deref(),
        );

        // Write substituted template to temp file
        let temp_file =
            std::env::temp_dir().join(format!("bumper_template_{}.mlt", Uuid::new_v4()));
        std::fs::write(&temp_file, substituted)?;

        // Ensure output directory exists
        let output_dir = PathBuf::from("static/media/bumpers");
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        // Generate output filename
        let output_filename = format!("{}.mp4", Uuid::new_v4());
        let output_path = output_dir.join(&output_filename);

        // Render with melt
        tracing::info!(
            "Rendering bumper '{}' to {}",
            bumper.name,
            output_path.display()
        );

        let status = Command::new("melt")
            .arg(&temp_file)
            .arg("-consumer")
            .arg(format!("avformat:{}", output_path.display()))
            .arg("vcodec=libx264")
            .arg("acodec=aac")
            .status()?;

        // Clean up temp file
        std::fs::remove_file(&temp_file)?;

        if !status.success() {
            return Err(anyhow!("melt command failed with status: {}", status));
        }

        // Extract duration
        let duration = self.get_duration_ms(&output_path)?;

        // Update database with rendered path and duration
        diesel::update(bumpers.filter(id.eq(Some(bumper_id))))
            .set((
                rendered_path.eq(Some(format!("media/bumpers/{}", output_filename))),
                duration_ms.eq(Some(duration)),
                last_rendered_at.eq(Some(chrono::Utc::now().naive_utc())),
            ))
            .execute(&mut conn)?;

        tracing::info!(
            "Bumper '{}' rendered successfully ({}ms)",
            bumper.name,
            duration
        );

        Ok(output_path)
    }

    /// Render all template bumpers
    pub fn render_all_templates(&self) -> Result<Vec<PathBuf>> {
        use crate::schema::bumpers::dsl::*;
        let mut conn = self.db.get()?;

        let template_bumpers: Vec<Bumper> = bumpers.filter(is_template.eq(true)).load(&mut conn)?;

        let mut rendered_paths = Vec::new();

        for bumper in template_bumpers {
            if let Some(bumper_id) = bumper.id {
                match self.render_template(bumper_id) {
                    Ok(path) => {
                        rendered_paths.push(path);
                    }
                    Err(e) => {
                        tracing::error!("Failed to render bumper {}: {}", bumper.name, e);
                    }
                }
            }
        }

        Ok(rendered_paths)
    }

    /// Render a bumper back MLT file to MP4
    pub fn render_bumper_back(&self, back_id: i32) -> Result<PathBuf> {
        use crate::schema::bumper_backs::dsl::*;
        let mut conn = self.db.get()?;

        // Get bumper back from database
        let back: BumperBack = bumper_backs.filter(id.eq(Some(back_id))).first(&mut conn)?;

        // Check if file_path ends with .mlt (MLT template) or is already a video
        if !back.file_path.ends_with(".mlt") {
            // Already a video file, no rendering needed
            return Ok(PathBuf::from(&back.file_path));
        }

        // Read MLT content from disk
        let mlt_file_path = PathBuf::from("static").join(&back.file_path);
        if !mlt_file_path.exists() {
            return Err(anyhow!(
                "Bumper back MLT file not found: {}",
                back.file_path
            ));
        }

        // Ensure output directory exists
        let output_dir = PathBuf::from("static/media/bumper_backs");
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        // Generate output filename
        let output_filename = format!("{}.mp4", Uuid::new_v4());
        let output_path = output_dir.join(&output_filename);

        // Render with melt
        tracing::info!(
            "Rendering bumper back '{}' to {}",
            back.name,
            output_path.display()
        );

        let status = Command::new("melt")
            .arg(&mlt_file_path)
            .arg("-consumer")
            .arg(format!("avformat:{}", output_path.display()))
            .arg("vcodec=libx264")
            .arg("acodec=aac")
            .status()?;

        if !status.success() {
            return Err(anyhow!("melt command failed with status: {}", status));
        }

        // Extract duration
        let duration = self.get_duration_ms(&output_path)?;

        // Update database with rendered path and duration
        use crate::schema::bumper_backs::dsl as bb_dsl;
        diesel::update(bb_dsl::bumper_backs.filter(bb_dsl::id.eq(Some(back_id))))
            .set((
                bb_dsl::file_path.eq(format!("media/bumper_backs/{}", output_filename)),
                bb_dsl::duration_ms.eq(Some(duration)),
            ))
            .execute(&mut conn)?;

        tracing::info!(
            "Bumper back '{}' rendered successfully ({}ms)",
            back.name,
            duration
        );

        Ok(output_path)
    }

    /// Render all bumper back MLT files
    pub fn render_all_bumper_backs(&self) -> Result<Vec<PathBuf>> {
        use crate::schema::bumper_backs::dsl::*;
        let mut conn = self.db.get()?;

        let backs: Vec<BumperBack> = bumper_backs.load(&mut conn)?;

        let mut rendered_paths = Vec::new();

        for back in backs {
            if back.file_path.ends_with(".mlt") {
                if let Some(back_id) = back.id {
                    match self.render_bumper_back(back_id) {
                        Ok(path) => {
                            rendered_paths.push(path);
                        }
                        Err(e) => {
                            tracing::error!("Failed to render bumper back {}: {}", back.name, e);
                        }
                    }
                }
            }
        }

        Ok(rendered_paths)
    }

    /// Process an uploaded file for a bumper back
    pub fn process_uploaded_file(&self, temp_path: &Path, filename: &str) -> Result<PathBuf> {
        // Ensure static/media/bumper_backs exists
        let output_dir = PathBuf::from("static/media/bumper_backs");
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        // Generate final filename (keeping extension or just using UUID)
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");

        let output_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let output_path = output_dir.join(&output_filename);

        // Move/Copy temp file to final location
        std::fs::copy(temp_path, &output_path)?;
        // Ideally we'd move/rename, but copy is safer across filesystems for temp vs static.
        // The caller (API handler) should clean up the temp file if it created one,
        // OR if we are passed a temp file path that we own, we can try rename first.
        // For simplicity with multipart upload streams, let's assume we copy the content.

        // Return relative path for DB
        Ok(PathBuf::from(format!(
            "media/bumper_backs/{}",
            output_filename
        )))
    }

    /// Download a remote file using curl
    pub fn download_remote_file(&self, url: &str) -> Result<PathBuf> {
        // Validate URL
        let parsed_url = url::Url::parse(url)?;

        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(anyhow!("Invalid URL scheme: {}", parsed_url.scheme()));
        }

        if let Some(host_str) = parsed_url.host_str() {
            if host_str == "localhost" {
                return Err(anyhow!("Access to localhost is denied"));
            }

            if let Ok(ip) = host_str.parse::<std::net::IpAddr>() {
                if ip.is_loopback() || ip.is_unspecified() {
                    return Err(anyhow!("Access to local/unspecified IP is denied"));
                }
                match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        if ipv4.is_private() || ipv4.is_link_local() {
                            return Err(anyhow!("Access to private network is denied"));
                        }
                    }
                    std::net::IpAddr::V6(ipv6) => {
                        // is_unique_local() is unstable, but fc00::/7 are ULA (private)
                        // fe80::/10 are link-local.
                        // For now just check loopback/unspecified which we did.
                        // We can manually check ranges if needed, but IPv6 private range support in std is limited.
                        if (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                            return Err(anyhow!("Access to private network is denied"));
                        }
                    }
                }
            }
        }

        // Ensure static/media/bumper_backs exists
        let output_dir = PathBuf::from("static/media/bumper_backs");
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        // Simple extension inference or default to mp4
        let extension = if url.ends_with(".webm") {
            "webm"
        } else if url.ends_with(".mov") {
            "mov"
        } else {
            "mp4"
        };

        let output_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let output_path = output_dir.join(&output_filename);

        tracing::info!(
            "Downloading bumper back from {} to {}",
            url,
            output_path.display()
        );

        // Use curl as requested
        let status = Command::new("curl")
            .arg("-L") // Follow redirects
            .arg("-f") // Fail silently (no output at all) on server errors
            .arg("-A") // User Agent
            .arg("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .arg("-o")
            .arg(&output_path)
            .arg(url)
            .status()?;

        if !status.success() {
            // Cleanup empty file if it exists
            if output_path.exists() {
                let _ = std::fs::remove_file(&output_path);
            }
            return Err(anyhow!("curl download failed with status: {}", status));
        }

        // Check if file has content
        let metadata = std::fs::metadata(&output_path)?;
        if metadata.len() == 0 {
            let _ = std::fs::remove_file(&output_path);
            return Err(anyhow!("Downloaded file is empty"));
        }

        // Return relative path for DB
        Ok(PathBuf::from(format!(
            "media/bumper_backs/{}",
            output_filename
        )))
    }

    pub fn get_duration_ms_public(&self, relative_path: &str) -> Result<i32> {
        let full_path = PathBuf::from("static").join(relative_path);
        self.get_duration_ms(&full_path)
    }
}
