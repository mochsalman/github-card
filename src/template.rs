use crate::config::UserConfig;
use crate::format::{calculate_uptime, format_power_number};
use crate::github::Stats;

/// Ganti semua placeholder `{{...}}` di template SVG dengan data stats & config user.
pub fn render_svg(template: &str, username: &str, stats: &Stats, config: &UserConfig) -> String {
    let uptime_display = if config.host.uptime == "-" {
        "-".to_string()
    } else {
        calculate_uptime(&config.host.uptime)
    };

    template
        .replace("{{username}}", username)
        .replace("{{repos}}", &stats.repos.to_string())
        .replace("{{stars}}", &stats.stars.to_string())
        .replace("{{commits}}", &stats.commits.to_string())
        .replace("{{lang_programming}}", &stats.top_languages)
        .replace("{{contributed}}", &stats.contributed.to_string())
        .replace("{{follower}}", &stats.followers.to_string())
        .replace("{{loc_data}}", &format_power_number(stats.loc_net()))
        .replace("{{loc_add}}", &format_power_number(stats.loc_add))
        .replace("{{loc_del}}", &format_power_number(stats.loc_del))
        .replace("{{uptime}}", &uptime_display)
        // field dari config user (preferences.toml)
        .replace("{{os}}", &config.host.os)
        .replace("{{uptime}}", &config.host.uptime)
        .replace("{{host}}", &config.host.host)
        .replace("{{kernel}}", &config.host.kernel)
        .replace("{{ide}}", &config.host.ide)
        .replace("{{lang_secondary}}", &config.languages.secondary)
        .replace("{{lang_native}}", &config.languages.native)
        .replace("{{softskill}}", &config.skills.softskill)
        .replace("{{hardskill}}", &config.skills.hardskill)
        .replace("{{email_personal}}", &config.contact.email.personal)
        .replace("{{email_work}}", &config.contact.email.work)
        .replace("{{linkedin}}", &config.contact.linked_in)
        .replace("{{discord}}", &config.contact.discord)
}
