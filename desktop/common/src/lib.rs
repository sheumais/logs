use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: UserInfo,
    pub enabled_features: EnabledFeatures,
    #[serde(rename = "guildSelectItems")]
    pub guild_select_items: Vec<GuildSelectInfo>,
    #[serde(rename = "reportVisibilitySelectItems")]
    pub report_visibility_select_items: Vec<LabelValue>,
    #[serde(rename = "regionOrServerSelectItems")]
    pub region_or_server_select_items: Vec<ValueLabel>,
    #[serde(rename = "reportTagSelectItems", default)]
    pub report_tag_select_items: Option<HashMap<String, Vec<LabelValue>>>,
}

impl LoginResponse {
    pub fn tags_for_guild(&self, guild_id: i32) -> Option<&Vec<LabelValue>> {
        if let Some(report_tags) = &self.report_tag_select_items {
            if guild_id < 0 {return None} else {
                report_tags.get(&guild_id.to_string())
            }
        } else {
            None
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: u32,
    #[serde(rename = "userName")]
    pub username: String,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    pub guilds: Vec<GuildInfo>,
    #[serde(default)]
    pub characters: Vec<CharacterInfo>,
    pub thumbnail: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnabledFeatures {
    #[serde(rename = "noAds")]
    pub no_ads: bool,
    #[serde(rename = "realTimeLiveLogging")]
    pub real_time_live_logging: bool,
    pub meters: bool,
    #[serde(rename = "liveFightData")]
    pub live_fight_data: bool,
    #[serde(rename = "tooltipAddon")]
    pub tooltip_addon: bool,
    #[serde(rename = "tooltipAddonTierTwoData")]
    pub tooltip_addon_tier_two_data: bool,
    #[serde(rename = "autoLog")]
    pub auto_log: bool,
    #[serde(rename = "metersLiveParse")]
    pub meters_live_parse: bool,
    #[serde(rename = "metersRaceTheGhost")]
    pub meters_race_the_ghost: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GuildInfo {
    pub id: u16,
    pub name: String,
    pub rank: u8,
    pub guild_logo: GuildLogo,
    pub faction: u8,
    #[serde(rename = "isRecruit")]
    pub is_recruit: bool,
    #[serde(rename = "isOfficer")]
    pub is_officer: bool,
    #[serde(rename = "isGuildMaster")]
    pub is_guild_master: bool,
    pub server: Server,
    pub region: Region,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GuildSelectInfo {
    pub value: i32, // -1 for personal logs
    pub label: String,
    pub logo: GuildLogo,
    #[serde(rename = "cssClassName")]
    pub css_class_name: String,
    #[serde(rename = "regionId")]
    pub region_id: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GuildLogo {
    pub url: String,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
    #[serde(rename = "fallbackUrl")]
    pub fallback_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterInfo {
    pub id: u32,
    pub name: String,
    #[serde(rename = "cssClassName")]
    pub class_name: String,
    pub thumbnail: String,
    pub server: Server,
    pub region: Region,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub id: u8,
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LabelValue {
    pub label: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ValueLabel {
    pub label: String,
    pub value: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EncounterReportCode {
    pub code: String
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UploadSettings {
    pub guild: i32,
    pub visibility: u8,
    pub region: u8,
    pub description: String,
    pub rewind: bool,
    pub tag: Option<i32>,
    pub remember_description: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UpdateInformation {
    pub version: String,
    pub current_version: String,
}
