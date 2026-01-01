use anyhow::{Context, bail};
use chrono::Utc;
use clap::Parser;
use regex_macro::regex;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::{env, process};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};
use twilight_http::Response;
use twilight_http::client::ClientBuilder;
use twilight_http::request::TryIntoRequest;
use twilight_http::response::marker::EmptyBody;
use twilight_mention::Mention;
use twilight_model::id::Id;
use twilight_model::id::marker::RoleMarker;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};
use twilight_util::link::webhook;

#[derive(Debug, Parser)]
#[clap(version, about, author)]
struct Inputs {
    // project info
    #[clap(long = "project-name")]
    project_name: String,
    #[clap(long = "project-version")]
    project_version: String,
    #[clap(long = "project-sourcecode-url")]
    project_sourcecode_url: String,

    // discord metadata
    #[clap(long = "discord-webhook-url")]
    discord_webhook_url: String,
    #[clap(long = "discord-thumbnail-url")]
    discord_thumbnail_url: String,
    #[clap(long = "discord-notification-role-id")]
    discord_notification_role_id: String,
    #[clap(long = "discord-ping-notification-role")]
    discord_ping_notification_role: String,

    // TODO maven/custom direct downloads?
    // publishing platforms
    #[clap(long = "curseforge-project-id")]
    curseforge_project_id: String,
    #[clap(long = "modrinth-project-id")]
    modrinth_project_id: String,

    // formatting
    #[clap(long = "sourcecode-emoji")]
    sourcecode_emoji: String,
    #[clap(long = "curseforge-emoji")]
    curseforge_emoji: String,
    #[clap(long = "modrinth-emoji")]
    modrinth_emoji: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    match wrapped_main().await {
        Ok(_) => {
            info!("Success");
        }
        Err(err) => {
            error!("ERROR: {:#}", err);
            process::exit(1);
        }
    }
}

async fn wrapped_main() -> anyhow::Result<()> {
    let github_output_path =
        env::var("GITHUB_OUTPUT").expect("GITHUB_OUTPUT environment variable not set");
    let args = Inputs::parse();

    let mut webhook_url = args.discord_webhook_url.as_str();

    if webhook_url.is_empty() {
        bail!("No webhook URL provided!");
    }

    if webhook_url.ends_with("/github") {
        warn!(
            "Webhook URL ends in /github, this is not going to work as intended. Removing suffix.."
        );
        webhook_url = &webhook_url[..webhook_url.len() - "/github".len()];
    }

    let (id, token) = webhook::parse(webhook_url)
        .with_context(|| format!("Failed to parse webhook URL: {webhook_url}"))?;
    let token = token.with_context(|| format!("webhook URL contained no token: {webhook_url}"))?;

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(30)) // default is too low for GH actions
        .build();

    let project_name = if args.project_name.is_empty() {
        let name = args
            .project_sourcecode_url
            .split('/')
            .next_back()
            .with_context(|| {
                format!(
                    "Failed to parse repository name from: {}",
                    args.project_sourcecode_url
                )
            })?;
        warn!("Project name not specified! Assuming '{name}' from sourcecode URL!");

        name
    } else {
        &args.project_name
    };

    let project_version = args.project_version;

    let mut description = vec![
        format!("# {project_name} {project_version}",),
        String::new(),
    ];

    let has_downloads_section =
        !args.curseforge_project_id.is_empty() || !args.modrinth_project_id.is_empty();

    if has_downloads_section {
        description.push("## Downloads:".to_string());
        let mut downloads = Vec::new();
        if !args.curseforge_project_id.is_empty() {
            let url = format!(
                "https://mods.cf/{project_id}",
                project_id = args.curseforge_project_id
            );
            downloads.push(build_prefixed_url(
                "Curseforge",
                &url,
                &args.curseforge_emoji,
            ));
        }
        if !args.modrinth_project_id.is_empty() {
            let url = format!(
                "https://modrinth.com/mod/{project_id}",
                project_id = args.modrinth_project_id
            );
            downloads.push(build_prefixed_url("Modrinth", &url, &args.modrinth_emoji));
        }
        description.push(downloads.join(" | "));
    }

    if !args.project_sourcecode_url.is_empty() {
        if has_downloads_section {
            description.push(String::new());
        }

        description.push(build_prefixed_url(
            "Source Code",
            &args.project_sourcecode_url,
            &args.sourcecode_emoji,
        ));
    }

    let mut embed_builder = EmbedBuilder::new()
        .color(0x8dcf88)
        .timestamp(Timestamp::from_micros(Utc::now().timestamp_micros())?)
        .description(description.join("\n"));

    if !args.discord_thumbnail_url.is_empty() {
        debug!("Adding thumbnail to embed: {}", args.discord_thumbnail_url);
        embed_builder =
            embed_builder.thumbnail(ImageSource::url(args.discord_thumbnail_url.as_str())?);
    }

    let embeds = vec![embed_builder.build()];

    const WEBHOOK_AVATAR_URL: &str = "https://avatars.githubusercontent.com/u/141473891?s=256";
    const WEBHOOK_USERNAME: &str = "Mod Updates";

    info!("Sending webhook message to Discord");
    let request = client
        .execute_webhook(id, token)
        .avatar_url(WEBHOOK_AVATAR_URL)
        .username(WEBHOOK_USERNAME)
        .embeds(&embeds)
        .try_into_request()?;

    let response: Response<EmptyBody> = client.request(request.clone()).await?;

    let mut out = File::create(github_output_path)?;
    writeln!(out, "response_status={}", response.status())?;

    // we can just use the raw body here since we're not adding any attachments
    if let Some(data) = request.body() {
        writeln!(out, "message<<EOF")?;
        out.write_all(data)?;
        writeln!(out)?;
        writeln!(out, "EOF")?;
    }

    let should_ping_role: bool = if args.discord_ping_notification_role.is_empty() {
        // default: analyze version and don't ping if it's a pre-release
        !regex!(r"[-+_](alpha)|(beta)|(rc)|(pre-?(release)?)|(snapshot)|(dev).*")
            .is_match(&project_version)
    } else {
        // treat any value other than "false" as true
        !args
            .discord_ping_notification_role
            .eq_ignore_ascii_case("false")
    };

    if should_ping_role {
        info!("Waiting 5 seconds before pinging notification role");
        sleep(Duration::from_secs(5)).await;

        info!("Pinging notification role");

        let mut role_str = args.discord_notification_role_id;
        if role_str.starts_with("<@&") && role_str.ends_with('>') {
            role_str = role_str[3..role_str.len() - 1].to_string();
        }

        let role_id: Id<RoleMarker> = role_str.parse()?;
        client
            .execute_webhook(id, token)
            .avatar_url(WEBHOOK_AVATAR_URL)
            .username(WEBHOOK_USERNAME)
            .content(role_id.mention().to_string().as_str())
            .await?;
    }

    Ok(())
}

fn build_prefixed_url(title: &str, url: &str, optional_emoji: &str) -> String {
    let mut result = String::new();
    if !optional_emoji.is_empty() {
        result.push_str(&optional_emoji);
        result.push(' ');
    }

    let link = format!("[{title}]({url})");
    result.push_str(&link);

    result
}
