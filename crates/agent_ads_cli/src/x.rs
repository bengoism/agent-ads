use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};

use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::secret_store::SecretStore;
use agent_ads_core::x_config::{
    x_inspect_auth, XAuthSnapshot, XConfigOverrides, XConfigSnapshot, XResolvedConfig,
    XSecretSource, XSecretStatus, X_DEFAULT_API_VERSION,
};
use agent_ads_core::x_endpoints::{account_scoped, accounts, analytics as x_analytics};
use agent_ads_core::{
    mutate_auth_bundle, XAuthBundle, XClient, XError, XResponse, AUTH_BUNDLE_ACCOUNT,
    AUTH_BUNDLE_SERVICE, X_ADS_ACCESS_TOKEN_ENV_VAR, X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
    X_ADS_CONSUMER_KEY_ENV_VAR, X_ADS_CONSUMER_SECRET_ENV_VAR,
};
use clap::{Args, Subcommand, ValueEnum};
use rpassword::prompt_password;
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, Duration as TimeDuration, OffsetDateTime};
use tokio::time::sleep;

use crate::{command_result, CommandResult};

macro_rules! list_get_subcommand {
    ($name:ident, $list_args:ty, $get_args:ty, $list_about:expr, $get_about:expr) => {
        #[derive(Subcommand, Debug)]
        pub enum $name {
            #[command(about = $list_about, visible_alias = "ls")]
            List($list_args),
            #[command(about = $get_about, visible_alias = "cat")]
            Get($get_args),
        }
    };
}

macro_rules! list_only_subcommand {
    ($name:ident, $list_args:ty, $list_about:expr) => {
        #[derive(Subcommand, Debug)]
        pub enum $name {
            #[command(about = $list_about, visible_alias = "ls")]
            List($list_args),
        }
    };
}

#[derive(Subcommand, Debug)]
pub enum XCommand {
    #[command(about = "List and inspect X ads accounts")]
    Accounts {
        #[command(subcommand)]
        command: AccountsCommand,
    },
    #[command(about = "Inspect the authenticated user's access to an X ads account")]
    AuthenticatedUserAccess {
        #[command(subcommand)]
        command: AuthenticatedUserAccessCommand,
    },
    #[command(about = "List and inspect campaigns")]
    Campaigns {
        #[command(subcommand)]
        command: CampaignsCommand,
    },
    #[command(about = "List and inspect line items")]
    LineItems {
        #[command(subcommand)]
        command: LineItemsCommand,
    },
    #[command(about = "List and inspect funding instruments")]
    FundingInstruments {
        #[command(subcommand)]
        command: FundingInstrumentsCommand,
    },
    #[command(about = "List and inspect promotable users")]
    PromotableUsers {
        #[command(subcommand)]
        command: PromotableUsersCommand,
    },
    #[command(about = "List and inspect promoted accounts")]
    PromotedAccounts {
        #[command(subcommand)]
        command: PromotedAccountsCommand,
    },
    #[command(about = "List and inspect promoted tweets")]
    PromotedTweets {
        #[command(subcommand)]
        command: PromotedTweetsCommand,
    },
    #[command(about = "List and inspect targeting criteria")]
    TargetingCriteria {
        #[command(subcommand)]
        command: TargetingCriteriaCommand,
    },
    #[command(about = "List and inspect account apps")]
    AccountApps {
        #[command(subcommand)]
        command: AccountAppsCommand,
    },
    #[command(about = "List and inspect account media")]
    AccountMedia {
        #[command(subcommand)]
        command: AccountMediaCommand,
    },
    #[command(about = "List and inspect media library assets")]
    MediaLibrary {
        #[command(subcommand)]
        command: MediaLibraryCommand,
    },
    #[command(about = "List and inspect cards")]
    Cards {
        #[command(subcommand)]
        command: CardsCommand,
    },
    #[command(about = "List and inspect draft tweets")]
    DraftTweets {
        #[command(subcommand)]
        command: DraftTweetsCommand,
    },
    #[command(about = "List and inspect scheduled tweets")]
    ScheduledTweets {
        #[command(subcommand)]
        command: ScheduledTweetsCommand,
    },
    #[command(about = "List promoted-only tweets in the scoped timeline")]
    ScopedTimeline {
        #[command(subcommand)]
        command: ScopedTimelineCommand,
    },
    #[command(about = "List and inspect custom audiences")]
    CustomAudiences {
        #[command(subcommand)]
        command: CustomAudiencesCommand,
    },
    #[command(about = "List and inspect do-not-reach lists")]
    DoNotReachLists {
        #[command(subcommand)]
        command: DoNotReachListsCommand,
    },
    #[command(about = "List and inspect web event tags")]
    WebEventTags {
        #[command(subcommand)]
        command: WebEventTagsCommand,
    },
    #[command(about = "List and inspect app lists")]
    AppLists {
        #[command(subcommand)]
        command: AppListsCommand,
    },
    #[command(about = "List and inspect AB tests")]
    AbTests {
        #[command(subcommand)]
        command: AbTestsCommand,
    },
    #[command(about = "Query X Ads analytics endpoints")]
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommand,
    },
    #[command(about = "Manage stored X Ads credentials")]
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    #[command(about = "Verify auth, config, and API connectivity")]
    Doctor(DoctorArgs),
    #[command(about = "Inspect and validate configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum AccountsCommand {
    #[command(about = "List accessible X ads accounts", visible_alias = "ls")]
    List(AccountsListArgs),
    #[command(about = "Get a single X ads account", visible_alias = "cat")]
    Get(AccountGetArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthenticatedUserAccessCommand {
    #[command(
        about = "Inspect authenticated user access for an account",
        visible_alias = "cat"
    )]
    Get(AuthenticatedUserAccessGetArgs),
}

list_get_subcommand!(
    CampaignsCommand,
    CampaignListArgs,
    CampaignGetArgs,
    "List campaigns for an ads account",
    "Get a single campaign"
);
list_get_subcommand!(
    LineItemsCommand,
    LineItemListArgs,
    LineItemGetArgs,
    "List line items for an ads account",
    "Get a single line item"
);
list_get_subcommand!(
    FundingInstrumentsCommand,
    FundingInstrumentListArgs,
    FundingInstrumentGetArgs,
    "List funding instruments for an ads account",
    "Get a single funding instrument"
);
list_get_subcommand!(
    PromotableUsersCommand,
    PromotableUserListArgs,
    PromotableUserGetArgs,
    "List promotable users for an ads account",
    "Get a single promotable user"
);
list_get_subcommand!(
    PromotedAccountsCommand,
    PromotedAccountListArgs,
    PromotedAccountGetArgs,
    "List promoted accounts for an ads account",
    "Get a single promoted account"
);
list_get_subcommand!(
    PromotedTweetsCommand,
    PromotedTweetListArgs,
    PromotedTweetGetArgs,
    "List promoted tweets for an ads account",
    "Get a single promoted tweet"
);
list_get_subcommand!(
    TargetingCriteriaCommand,
    TargetingCriterionListArgs,
    TargetingCriterionGetArgs,
    "List targeting criteria for an ads account",
    "Get a single targeting criterion"
);
list_get_subcommand!(
    AccountAppsCommand,
    AccountAppListArgs,
    AccountAppGetArgs,
    "List account apps for an ads account",
    "Get a single account app"
);
list_get_subcommand!(
    AccountMediaCommand,
    AccountMediaListArgs,
    AccountMediaGetArgs,
    "List account media for an ads account",
    "Get a single account media asset"
);
list_get_subcommand!(
    MediaLibraryCommand,
    MediaLibraryListArgs,
    MediaLibraryGetArgs,
    "List media library assets for an ads account",
    "Get a single media library asset"
);
list_get_subcommand!(
    CardsCommand,
    CardListArgs,
    CardGetArgs,
    "List cards for an ads account",
    "Get a single card"
);
list_get_subcommand!(
    DraftTweetsCommand,
    DraftTweetListArgs,
    DraftTweetGetArgs,
    "List draft tweets for an ads account",
    "Get a single draft tweet"
);
list_get_subcommand!(
    ScheduledTweetsCommand,
    ScheduledTweetListArgs,
    ScheduledTweetGetArgs,
    "List scheduled tweets for an ads account",
    "Get a single scheduled tweet"
);
list_only_subcommand!(
    ScopedTimelineCommand,
    ScopedTimelineListArgs,
    "List promoted-only tweets for an ads account"
);
list_get_subcommand!(
    CustomAudiencesCommand,
    CustomAudienceListArgs,
    CustomAudienceGetArgs,
    "List custom audiences for an ads account",
    "Get a single custom audience"
);
list_get_subcommand!(
    DoNotReachListsCommand,
    DoNotReachListListArgs,
    DoNotReachListGetArgs,
    "List do-not-reach lists for an ads account",
    "Get a single do-not-reach list"
);
list_get_subcommand!(
    WebEventTagsCommand,
    WebEventTagListArgs,
    WebEventTagGetArgs,
    "List web event tags for an ads account",
    "Get a single web event tag"
);
list_get_subcommand!(
    AppListsCommand,
    AppListListArgs,
    AppListGetArgs,
    "List app lists for an ads account",
    "Get a single app list"
);
list_get_subcommand!(
    AbTestsCommand,
    AbTestListArgs,
    AbTestGetArgs,
    "List AB tests for an ads account",
    "Get a single AB test"
);

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommand {
    #[command(about = "Run a synchronous analytics query")]
    Query(AnalyticsQueryArgs),
    #[command(about = "Run a reach and average-frequency query")]
    Reach(ReachQueryArgs),
    #[command(about = "List active entities for analytics syncs")]
    ActiveEntities(ActiveEntitiesArgs),
    #[command(about = "Manage async analytics jobs")]
    Jobs {
        #[command(subcommand)]
        command: AnalyticsJobsCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsJobsCommand {
    #[command(about = "Submit an async analytics job")]
    Submit(AnalyticsJobSubmitArgs),
    #[command(about = "Check async analytics job status")]
    Status(AnalyticsJobStatusArgs),
    #[command(about = "Poll until an async analytics job reaches a terminal state")]
    Wait(AnalyticsJobWaitArgs),
    #[command(about = "Download the completed async analytics results")]
    Download(AnalyticsJobDownloadArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Store X Ads credentials in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete stored X Ads credentials")]
    Delete,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    #[command(about = "Show resolved config file path")]
    Path,
    #[command(about = "Show full resolved configuration")]
    Show,
    #[command(about = "Validate config file")]
    Validate,
}

#[derive(Args, Debug, Clone, Default)]
pub struct XPaginationArgs {
    #[arg(long, help = "Resume from an X cursor")]
    pub cursor: Option<String>,
    #[arg(long = "page-size", help = "Items per API request")]
    pub page_size: Option<u32>,
    #[arg(long, help = "Auto-follow all available pages")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total items")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct XCollectionArgs {
    #[command(flatten)]
    pub pagination: XPaginationArgs,
    #[arg(long = "sort-by", help = "Provider-native sort key")]
    pub sort_by: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AccountSelectorArgs {
    #[arg(long = "account-id", help = "X ads account ID")]
    pub account_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AccountsListArgs {
    #[arg(
        long = "account-id",
        value_delimiter = ',',
        help = "Filter to one or more account IDs"
    )]
    pub account_ids: Vec<String>,
    #[arg(long = "with-deleted", help = "Include deleted accounts if supported")]
    pub with_deleted: bool,
    #[command(flatten)]
    pub pagination: XPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AccountGetArgs {
    #[arg(long = "account-id", help = "X ads account ID")]
    pub account_id: String,
    #[arg(long = "with-deleted", help = "Include deleted accounts if supported")]
    pub with_deleted: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthenticatedUserAccessGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter campaign IDs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "funding-instrument-id",
        value_delimiter = ',',
        help = "Filter funding instrument IDs"
    )]
    pub funding_instrument_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "campaign-id", help = "Campaign ID")]
    pub campaign_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct LineItemListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "line-item-id",
        value_delimiter = ',',
        help = "Filter line item IDs"
    )]
    pub line_item_ids: Vec<String>,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter campaign IDs"
    )]
    pub campaign_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct LineItemGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "line-item-id", help = "Line item ID")]
    pub line_item_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct FundingInstrumentListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "funding-instrument-id",
        value_delimiter = ',',
        help = "Filter funding instrument IDs"
    )]
    pub funding_instrument_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct FundingInstrumentGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "funding-instrument-id", help = "Funding instrument ID")]
    pub funding_instrument_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct PromotableUserListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "user-id", value_delimiter = ',', help = "Filter user IDs")]
    pub user_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PromotableUserGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "user-id", help = "Promotable user ID")]
    pub user_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct PromotedAccountListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "promoted-account-id",
        value_delimiter = ',',
        help = "Filter promoted account IDs"
    )]
    pub promoted_account_ids: Vec<String>,
    #[arg(
        long = "line-item-id",
        value_delimiter = ',',
        help = "Filter line item IDs"
    )]
    pub line_item_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PromotedAccountGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "promoted-account-id", help = "Promoted account ID")]
    pub promoted_account_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct PromotedTweetListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "promoted-tweet-id",
        value_delimiter = ',',
        help = "Filter promoted tweet IDs"
    )]
    pub promoted_tweet_ids: Vec<String>,
    #[arg(
        long = "line-item-id",
        value_delimiter = ',',
        help = "Filter line item IDs"
    )]
    pub line_item_ids: Vec<String>,
    #[arg(long = "tweet-id", value_delimiter = ',', help = "Filter tweet IDs")]
    pub tweet_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PromotedTweetGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "promoted-tweet-id", help = "Promoted tweet ID")]
    pub promoted_tweet_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct TargetingCriterionListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "targeting-criterion-id",
        value_delimiter = ',',
        help = "Filter targeting criterion IDs"
    )]
    pub targeting_criterion_ids: Vec<String>,
    #[arg(
        long = "line-item-id",
        value_delimiter = ',',
        help = "Filter line item IDs"
    )]
    pub line_item_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct TargetingCriterionGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "targeting-criterion-id", help = "Targeting criterion ID")]
    pub targeting_criterion_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct AccountAppListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "account-app-id",
        value_delimiter = ',',
        help = "Filter account app IDs"
    )]
    pub account_app_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AccountAppGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "account-app-id", help = "Account app ID")]
    pub account_app_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct AccountMediaListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "media-key", value_delimiter = ',', help = "Filter media keys")]
    pub media_keys: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AccountMediaGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "media-key", help = "Account media key")]
    pub media_key: String,
}

#[derive(Args, Debug, Clone)]
pub struct MediaLibraryListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "media-key", value_delimiter = ',', help = "Filter media keys")]
    pub media_keys: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct MediaLibraryGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "media-key", help = "Media library key")]
    pub media_key: String,
}

#[derive(Args, Debug, Clone)]
pub struct CardListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "card-uri", value_delimiter = ',', help = "Filter card URIs")]
    pub card_uris: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CardGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "card-uri", help = "Card URI")]
    pub card_uri: String,
}

#[derive(Args, Debug, Clone)]
pub struct DraftTweetListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "tweet-id", value_delimiter = ',', help = "Filter tweet IDs")]
    pub tweet_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DraftTweetGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "tweet-id", help = "Draft tweet ID")]
    pub tweet_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct ScheduledTweetListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "scheduled-tweet-id",
        value_delimiter = ',',
        help = "Filter scheduled tweet IDs"
    )]
    pub scheduled_tweet_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ScheduledTweetGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "scheduled-tweet-id", help = "Scheduled tweet ID")]
    pub scheduled_tweet_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct ScopedTimelineListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "tweet-id", value_delimiter = ',', help = "Filter tweet IDs")]
    pub tweet_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CustomAudienceListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "custom-audience-id",
        value_delimiter = ',',
        help = "Filter custom audience IDs"
    )]
    pub custom_audience_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CustomAudienceGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "custom-audience-id", help = "Custom audience ID")]
    pub custom_audience_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct DoNotReachListListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "do-not-reach-list-id",
        value_delimiter = ',',
        help = "Filter do-not-reach list IDs"
    )]
    pub do_not_reach_list_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DoNotReachListGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "do-not-reach-list-id", help = "Do-not-reach list ID")]
    pub do_not_reach_list_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct WebEventTagListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "web-event-tag-id",
        value_delimiter = ',',
        help = "Filter web event tag IDs"
    )]
    pub web_event_tag_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct WebEventTagGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "web-event-tag-id", help = "Web event tag ID")]
    pub web_event_tag_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct AppListListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "app-list-id",
        value_delimiter = ',',
        help = "Filter app list IDs"
    )]
    pub app_list_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AppListGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "app-list-id", help = "App list ID")]
    pub app_list_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct AbTestListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "ab-test-id",
        value_delimiter = ',',
        help = "Filter AB test IDs"
    )]
    pub ab_test_ids: Vec<String>,
    #[command(flatten)]
    pub collection: XCollectionArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AbTestGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "ab-test-id", help = "AB test ID")]
    pub ab_test_id: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AnalyticsEntityArg {
    Account,
    Campaign,
    FundingInstrument,
    LineItem,
    OrganicTweet,
    PromotedAccount,
    PromotedTweet,
    MediaCreative,
}

impl AnalyticsEntityArg {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Account => "ACCOUNT",
            Self::Campaign => "CAMPAIGN",
            Self::FundingInstrument => "FUNDING_INSTRUMENT",
            Self::LineItem => "LINE_ITEM",
            Self::OrganicTweet => "ORGANIC_TWEET",
            Self::PromotedAccount => "PROMOTED_ACCOUNT",
            Self::PromotedTweet => "PROMOTED_TWEET",
            Self::MediaCreative => "MEDIA_CREATIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ActiveEntityArg {
    Campaign,
    FundingInstrument,
    LineItem,
    MediaCreative,
    PromotedAccount,
    PromotedTweet,
}

impl ActiveEntityArg {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Campaign => "CAMPAIGN",
            Self::FundingInstrument => "FUNDING_INSTRUMENT",
            Self::LineItem => "LINE_ITEM",
            Self::MediaCreative => "MEDIA_CREATIVE",
            Self::PromotedAccount => "PROMOTED_ACCOUNT",
            Self::PromotedTweet => "PROMOTED_TWEET",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AnalyticsGranularityArg {
    Day,
    Hour,
    Total,
}

impl AnalyticsGranularityArg {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Day => "DAY",
            Self::Hour => "HOUR",
            Self::Total => "TOTAL",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PlacementArg {
    #[value(name = "all-on-twitter")]
    AllOnTwitter,
    #[value(name = "publisher-network")]
    PublisherNetwork,
    Spotlight,
    Trend,
}

impl PlacementArg {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::AllOnTwitter => "ALL_ON_TWITTER",
            Self::PublisherNetwork => "PUBLISHER_NETWORK",
            Self::Spotlight => "SPOTLIGHT",
            Self::Trend => "TREND",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReachLevelArg {
    Campaigns,
    #[value(name = "funding-instruments")]
    FundingInstruments,
}

impl ReachLevelArg {
    fn path(self) -> &'static str {
        match self {
            Self::Campaigns => "campaigns",
            Self::FundingInstruments => "funding_instruments",
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsQueryArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long, value_enum, help = "Analytics entity type")]
    pub entity: AnalyticsEntityArg,
    #[arg(
        long = "entity-id",
        value_delimiter = ',',
        help = "One to twenty entity IDs"
    )]
    pub entity_ids: Vec<String>,
    #[arg(long = "start-time", help = "Start time (RFC 3339, whole hour)")]
    pub start_time: String,
    #[arg(long = "end-time", help = "End time (RFC 3339, whole hour)")]
    pub end_time: String,
    #[arg(long, value_enum, help = "Analytics granularity")]
    pub granularity: AnalyticsGranularityArg,
    #[arg(long, value_enum, help = "Placement to query")]
    pub placement: PlacementArg,
    #[arg(long = "metric-group", value_delimiter = ',', help = "Metric groups")]
    pub metric_groups: Vec<String>,
    #[arg(long, help = "Optional country filter")]
    pub country: Option<String>,
    #[arg(long, help = "Optional platform filter")]
    pub platform: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ReachQueryArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long, value_enum, help = "Reach endpoint to query")]
    pub level: ReachLevelArg,
    #[arg(
        long = "id",
        value_delimiter = ',',
        help = "One to twenty resource IDs"
    )]
    pub ids: Vec<String>,
    #[arg(long = "start-time", help = "Start time (RFC 3339, whole hour)")]
    pub start_time: String,
    #[arg(long = "end-time", help = "End time (RFC 3339, whole hour)")]
    pub end_time: String,
}

#[derive(Args, Debug, Clone)]
pub struct ActiveEntitiesArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long, value_enum, help = "Entity type to inspect")]
    pub entity: ActiveEntityArg,
    #[arg(long = "start-time", help = "Start time (RFC 3339, whole hour)")]
    pub start_time: String,
    #[arg(long = "end-time", help = "End time (RFC 3339, whole hour)")]
    pub end_time: String,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter campaign IDs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "funding-instrument-id",
        value_delimiter = ',',
        help = "Filter funding instrument IDs"
    )]
    pub funding_instrument_ids: Vec<String>,
    #[arg(
        long = "line-item-id",
        value_delimiter = ',',
        help = "Filter line item IDs"
    )]
    pub line_item_ids: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsJobSubmitArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long, value_enum, help = "Analytics entity type")]
    pub entity: AnalyticsEntityArg,
    #[arg(
        long = "entity-id",
        value_delimiter = ',',
        help = "One to twenty entity IDs"
    )]
    pub entity_ids: Vec<String>,
    #[arg(long = "start-time", help = "Start time (RFC 3339, whole hour)")]
    pub start_time: String,
    #[arg(long = "end-time", help = "End time (RFC 3339, whole hour)")]
    pub end_time: String,
    #[arg(long, value_enum, help = "Analytics granularity")]
    pub granularity: AnalyticsGranularityArg,
    #[arg(long, value_enum, help = "Placement to query")]
    pub placement: PlacementArg,
    #[arg(long = "metric-group", value_delimiter = ',', help = "Metric groups")]
    pub metric_groups: Vec<String>,
    #[arg(long = "segmentation-type", help = "Optional segmentation type")]
    pub segmentation_type: Option<String>,
    #[arg(long, help = "Optional country filter")]
    pub country: Option<String>,
    #[arg(long, help = "Optional platform filter")]
    pub platform: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsJobStatusArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "job-id",
        value_delimiter = ',',
        help = "Filter one or more job IDs"
    )]
    pub job_ids: Vec<String>,
    #[command(flatten)]
    pub pagination: XPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsJobWaitArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "job-id", help = "Job ID to poll")]
    pub job_id: String,
    #[arg(long = "poll-interval-seconds", default_value_t = 5)]
    pub poll_interval_seconds: u64,
    #[arg(long = "timeout-seconds", default_value_t = 600)]
    pub timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsJobDownloadArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "job-id", help = "Job ID to download")]
    pub job_id: String,
    #[arg(long, help = "Wait until the job completes before downloading")]
    pub wait: bool,
    #[arg(long = "poll-interval-seconds", default_value_t = 5)]
    pub poll_interval_seconds: u64,
    #[arg(long = "timeout-seconds", default_value_t = 600)]
    pub timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    #[arg(long, help = "Also make a lightweight X Ads API request")]
    pub api: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthSetArgs {
    #[arg(
        long,
        conflicts_with_all = [
            "consumer_key",
            "consumer_secret",
            "access_token",
            "access_token_secret"
        ],
        help = "Read consumer key, consumer secret, access token, and access token secret from stdin"
    )]
    pub stdin: bool,
    #[arg(long = "consumer-key", help = "X Ads consumer key")]
    pub consumer_key: Option<String>,
    #[arg(long = "consumer-secret", help = "X Ads consumer secret")]
    pub consumer_secret: Option<String>,
    #[arg(long = "access-token", help = "X Ads access token")]
    pub access_token: Option<String>,
    #[arg(long = "access-token-secret", help = "X Ads access token secret")]
    pub access_token_secret: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct XAuthInputs {
    pub(crate) consumer_key: String,
    pub(crate) consumer_secret: String,
    pub(crate) access_token: String,
    pub(crate) access_token_secret: String,
}

pub fn handle_auth(
    command: AuthCommand,
    secret_store: &dyn SecretStore,
) -> Result<CommandResult, XError> {
    match command {
        AuthCommand::Set(args) => {
            let inputs = resolve_auth_inputs(&args)?;
            let outcome = mutate_auth_bundle(secret_store, move |bundle| {
                bundle.x = Some(XAuthBundle {
                    consumer_key: Some(inputs.consumer_key),
                    consumer_secret: Some(inputs.consumer_secret),
                    access_token: Some(inputs.access_token),
                    access_token_secret: Some(inputs.access_token_secret),
                });
            })
            .map_err(|error| auth_storage_error("store X Ads credentials", &error))?;

            Ok(x_command_result(
                json!({
                    "provider": "x",
                    "stored": true,
                    "recovered_invalid_bundle": outcome.recovered_invalid_bundle,
                    "credentials_stored": [
                        "consumer_key",
                        "consumer_secret",
                        "access_token",
                        "access_token_secret"
                    ],
                }),
                "/x/auth/set",
                0,
            ))
        }
        AuthCommand::Status => Ok(x_command_result(
            x_auth_status_payload(x_inspect_auth(secret_store)),
            "/x/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let mut deleted_consumer_key = false;
            let mut deleted_consumer_secret = false;
            let mut deleted_access_token = false;
            let mut deleted_access_token_secret = false;
            let outcome = mutate_auth_bundle(secret_store, |bundle| {
                let deleted_x = bundle.x.take();
                deleted_consumer_key = deleted_x
                    .as_ref()
                    .and_then(|x| x.consumer_key.as_ref())
                    .is_some();
                deleted_consumer_secret = deleted_x
                    .as_ref()
                    .and_then(|x| x.consumer_secret.as_ref())
                    .is_some();
                deleted_access_token = deleted_x
                    .as_ref()
                    .and_then(|x| x.access_token.as_ref())
                    .is_some();
                deleted_access_token_secret = deleted_x
                    .as_ref()
                    .and_then(|x| x.access_token_secret.as_ref())
                    .is_some();
            })
            .map_err(|error| auth_storage_error("delete X Ads credentials", &error))?;

            Ok(x_command_result(
                json!({
                    "provider": "x",
                    "consumer_key_deleted": deleted_consumer_key,
                    "consumer_secret_deleted": deleted_consumer_secret,
                    "access_token_deleted": deleted_access_token,
                    "access_token_secret_deleted": deleted_access_token_secret,
                    "recovered_invalid_bundle": outcome.recovered_invalid_bundle,
                }),
                "/x/auth/delete",
                0,
            ))
        }
    }
}

pub fn handle_config(
    command: ConfigCommand,
    snapshot: XConfigSnapshot,
) -> Result<CommandResult, XError> {
    match command {
        ConfigCommand::Path => Ok(x_command_result(
            json!({
                "path": snapshot.config_path,
                "exists": snapshot.config_file_exists,
            }),
            "/x/config/path",
            0,
        )),
        ConfigCommand::Show => Ok(x_command_result(json!(snapshot), "/x/config/show", 0)),
        ConfigCommand::Validate => Ok(x_command_result(
            json!({
                "valid": true,
                "config": snapshot,
            }),
            "/x/config/validate",
            0,
        )),
    }
}

pub async fn handle_doctor(
    args: DoctorArgs,
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &XConfigOverrides,
    snapshot: XConfigSnapshot,
) -> Result<CommandResult, XError> {
    let mut checks = vec![
        json!({
            "name": "credential_store",
            "ok": credential_store_check_ok(&snapshot),
            "detail": credential_store_detail(&snapshot),
        }),
        json!({
            "name": "config_file",
            "ok": snapshot.config_file_exists,
            "detail": if snapshot.config_file_exists {
                format!("using {}", snapshot.config_path.display())
            } else {
                format!("config file not found at {}", snapshot.config_path.display())
            }
        }),
        json!({
            "name": "consumer_key",
            "ok": snapshot.auth.consumer_key.present,
            "detail": secret_detail(X_ADS_CONSUMER_KEY_ENV_VAR, "consumer key", &snapshot.auth.consumer_key),
        }),
        json!({
            "name": "consumer_secret",
            "ok": snapshot.auth.consumer_secret.present,
            "detail": secret_detail(X_ADS_CONSUMER_SECRET_ENV_VAR, "consumer secret", &snapshot.auth.consumer_secret),
        }),
        json!({
            "name": "access_token",
            "ok": snapshot.auth.access_token.present,
            "detail": secret_detail(X_ADS_ACCESS_TOKEN_ENV_VAR, "access token", &snapshot.auth.access_token),
        }),
        json!({
            "name": "access_token_secret",
            "ok": snapshot.auth.access_token_secret.present,
            "detail": secret_detail(X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR, "access token secret", &snapshot.auth.access_token_secret),
        }),
    ];

    let mut ok = required_credentials_present(&snapshot.auth);
    if args.api {
        if ok {
            match XResolvedConfig::load(config_path, secret_store, overrides) {
                Ok(config) => match XClient::from_config(&config) {
                    Ok(client) => match accounts::list_accounts(
                        &client,
                        &[],
                        None,
                        Some(1),
                        None,
                        false,
                        Some(1),
                    )
                    .await
                    {
                        Ok(response) => {
                            let count = response
                                .data
                                .as_array()
                                .map(|items| items.len())
                                .unwrap_or(0);
                            checks.push(json!({
                                "name": "api_ping",
                                "ok": true,
                                "detail": format!("credentials accepted by X Ads API; sampled {} account record(s)", count),
                            }));
                        }
                        Err(error) => {
                            ok = false;
                            checks.push(json!({
                                "name": "api_ping",
                                "ok": false,
                                "detail": error.to_string(),
                            }));
                        }
                    },
                    Err(error) => {
                        ok = false;
                        checks.push(json!({
                            "name": "api_ping",
                            "ok": false,
                            "detail": error.to_string(),
                        }));
                    }
                },
                Err(error) => {
                    ok = false;
                    checks.push(json!({
                        "name": "api_ping",
                        "ok": false,
                        "detail": error.to_string(),
                    }));
                }
            }
        } else {
            ok = false;
            checks.push(json!({
                "name": "api_ping",
                "ok": false,
                "detail": "skipped because required X Ads credentials are missing",
            }));
        }
    }

    Ok(x_command_result(
        json!({
            "ok": ok,
            "checks": checks,
            "config": snapshot,
        }),
        "/x/doctor",
        if ok { 0 } else { 1 },
    ))
}

pub async fn dispatch_x_with_client(
    client: &XClient,
    config: &XResolvedConfig,
    command: XCommand,
) -> Result<CommandResult, XError> {
    match command {
        XCommand::Accounts { command } => match command {
            AccountsCommand::List(args) => {
                let response = accounts::list_accounts(
                    client,
                    &args.account_ids,
                    args.pagination.cursor.as_deref(),
                    args.pagination.page_size,
                    args.with_deleted.then_some(true),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(x_result(client, response, "/x/accounts/list", None, vec![]))
            }
            AccountsCommand::Get(args) => {
                let response = accounts::get_account(
                    client,
                    &args.account_id,
                    args.with_deleted.then_some(true),
                )
                .await?;
                Ok(x_result(
                    client,
                    response,
                    &format!("/x/accounts/{}", args.account_id),
                    Some(args.account_id),
                    vec![],
                ))
            }
        },
        XCommand::AuthenticatedUserAccess { command } => match command {
            AuthenticatedUserAccessCommand::Get(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let response = accounts::get_authenticated_user_access(client, &account_id).await?;
                Ok(x_result(
                    client,
                    response,
                    &format!("/x/accounts/{account_id}/authenticated_user_access"),
                    Some(account_id),
                    vec![],
                ))
            }
        },
        XCommand::Campaigns { command } => match command {
            CampaignsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "campaigns",
                    "/x/campaigns/list",
                    &args.collection,
                    &[
                        ("campaign_ids", &args.campaign_ids),
                        ("funding_instrument_ids", &args.funding_instrument_ids),
                    ],
                )
                .await
            }
            CampaignsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "campaigns",
                    &args.campaign_id,
                    "/x/campaigns/get",
                )
                .await
            }
        },
        XCommand::LineItems { command } => match command {
            LineItemsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "line_items",
                    "/x/line-items/list",
                    &args.collection,
                    &[
                        ("line_item_ids", &args.line_item_ids),
                        ("campaign_ids", &args.campaign_ids),
                    ],
                )
                .await
            }
            LineItemsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "line_items",
                    &args.line_item_id,
                    "/x/line-items/get",
                )
                .await
            }
        },
        XCommand::FundingInstruments { command } => match command {
            FundingInstrumentsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "funding_instruments",
                    "/x/funding-instruments/list",
                    &args.collection,
                    &[("funding_instrument_ids", &args.funding_instrument_ids)],
                )
                .await
            }
            FundingInstrumentsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "funding_instruments",
                    &args.funding_instrument_id,
                    "/x/funding-instruments/get",
                )
                .await
            }
        },
        XCommand::PromotableUsers { command } => match command {
            PromotableUsersCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "promotable_users",
                    "/x/promotable-users/list",
                    &args.collection,
                    &[("user_ids", &args.user_ids)],
                )
                .await
            }
            PromotableUsersCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "promotable_users",
                    &args.user_id,
                    "/x/promotable-users/get",
                )
                .await
            }
        },
        XCommand::PromotedAccounts { command } => match command {
            PromotedAccountsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "promoted_accounts",
                    "/x/promoted-accounts/list",
                    &args.collection,
                    &[
                        ("promoted_account_ids", &args.promoted_account_ids),
                        ("line_item_ids", &args.line_item_ids),
                    ],
                )
                .await
            }
            PromotedAccountsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "promoted_accounts",
                    &args.promoted_account_id,
                    "/x/promoted-accounts/get",
                )
                .await
            }
        },
        XCommand::PromotedTweets { command } => match command {
            PromotedTweetsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "promoted_tweets",
                    "/x/promoted-tweets/list",
                    &args.collection,
                    &[
                        ("promoted_tweet_ids", &args.promoted_tweet_ids),
                        ("line_item_ids", &args.line_item_ids),
                        ("tweet_ids", &args.tweet_ids),
                    ],
                )
                .await
            }
            PromotedTweetsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "promoted_tweets",
                    &args.promoted_tweet_id,
                    "/x/promoted-tweets/get",
                )
                .await
            }
        },
        XCommand::TargetingCriteria { command } => match command {
            TargetingCriteriaCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "targeting_criteria",
                    "/x/targeting-criteria/list",
                    &args.collection,
                    &[
                        ("targeting_criterion_ids", &args.targeting_criterion_ids),
                        ("line_item_ids", &args.line_item_ids),
                    ],
                )
                .await
            }
            TargetingCriteriaCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "targeting_criteria",
                    &args.targeting_criterion_id,
                    "/x/targeting-criteria/get",
                )
                .await
            }
        },
        XCommand::AccountApps { command } => match command {
            AccountAppsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "account_apps",
                    "/x/account-apps/list",
                    &args.collection,
                    &[("account_app_ids", &args.account_app_ids)],
                )
                .await
            }
            AccountAppsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "account_apps",
                    &args.account_app_id,
                    "/x/account-apps/get",
                )
                .await
            }
        },
        XCommand::AccountMedia { command } => match command {
            AccountMediaCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "account_media",
                    "/x/account-media/list",
                    &args.collection,
                    &[("media_keys", &args.media_keys)],
                )
                .await
            }
            AccountMediaCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "account_media",
                    &args.media_key,
                    "/x/account-media/get",
                )
                .await
            }
        },
        XCommand::MediaLibrary { command } => match command {
            MediaLibraryCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "media_library",
                    "/x/media-library/list",
                    &args.collection,
                    &[("media_keys", &args.media_keys)],
                )
                .await
            }
            MediaLibraryCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "media_library",
                    &args.media_key,
                    "/x/media-library/get",
                )
                .await
            }
        },
        XCommand::Cards { command } => match command {
            CardsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "cards",
                    "/x/cards/list",
                    &args.collection,
                    &[("card_uris", &args.card_uris)],
                )
                .await
            }
            CardsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "cards",
                    &args.card_uri,
                    "/x/cards/get",
                )
                .await
            }
        },
        XCommand::DraftTweets { command } => match command {
            DraftTweetsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "draft_tweets",
                    "/x/draft-tweets/list",
                    &args.collection,
                    &[("tweet_ids", &args.tweet_ids)],
                )
                .await
            }
            DraftTweetsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "draft_tweets",
                    &args.tweet_id,
                    "/x/draft-tweets/get",
                )
                .await
            }
        },
        XCommand::ScheduledTweets { command } => match command {
            ScheduledTweetsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "scheduled_tweets",
                    "/x/scheduled-tweets/list",
                    &args.collection,
                    &[("scheduled_tweet_ids", &args.scheduled_tweet_ids)],
                )
                .await
            }
            ScheduledTweetsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "scheduled_tweets",
                    &args.scheduled_tweet_id,
                    "/x/scheduled-tweets/get",
                )
                .await
            }
        },
        XCommand::ScopedTimeline { command } => match command {
            ScopedTimelineCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "scoped_timeline",
                    "/x/scoped-timeline/list",
                    &args.collection,
                    &[("tweet_ids", &args.tweet_ids)],
                )
                .await
            }
        },
        XCommand::CustomAudiences { command } => match command {
            CustomAudiencesCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "custom_audiences",
                    "/x/custom-audiences/list",
                    &args.collection,
                    &[("custom_audience_ids", &args.custom_audience_ids)],
                )
                .await
            }
            CustomAudiencesCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "custom_audiences",
                    &args.custom_audience_id,
                    "/x/custom-audiences/get",
                )
                .await
            }
        },
        XCommand::DoNotReachLists { command } => match command {
            DoNotReachListsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "do_not_reach_lists",
                    "/x/do-not-reach-lists/list",
                    &args.collection,
                    &[("do_not_reach_list_ids", &args.do_not_reach_list_ids)],
                )
                .await
            }
            DoNotReachListsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "do_not_reach_lists",
                    &args.do_not_reach_list_id,
                    "/x/do-not-reach-lists/get",
                )
                .await
            }
        },
        XCommand::WebEventTags { command } => match command {
            WebEventTagsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "web_event_tags",
                    "/x/web-event-tags/list",
                    &args.collection,
                    &[("web_event_tag_ids", &args.web_event_tag_ids)],
                )
                .await
            }
            WebEventTagsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "web_event_tags",
                    &args.web_event_tag_id,
                    "/x/web-event-tags/get",
                )
                .await
            }
        },
        XCommand::AppLists { command } => match command {
            AppListsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "app_lists",
                    "/x/app-lists/list",
                    &args.collection,
                    &[("app_list_ids", &args.app_list_ids)],
                )
                .await
            }
            AppListsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "app_lists",
                    &args.app_list_id,
                    "/x/app-lists/get",
                )
                .await
            }
        },
        XCommand::AbTests { command } => match command {
            AbTestsCommand::List(args) => {
                list_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "ab_tests",
                    "/x/ab-tests/list",
                    &args.collection,
                    &[("ab_test_ids", &args.ab_test_ids)],
                )
                .await
            }
            AbTestsCommand::Get(args) => {
                get_account_resource(
                    client,
                    config,
                    args.selector.account_id.as_deref(),
                    "ab_tests",
                    &args.ab_test_id,
                    "/x/ab-tests/get",
                )
                .await
            }
        },
        XCommand::Analytics { command } => match command {
            AnalyticsCommand::Query(args) => dispatch_sync_analytics(client, config, args).await,
            AnalyticsCommand::Reach(args) => dispatch_reach_query(client, config, args).await,
            AnalyticsCommand::ActiveEntities(args) => {
                dispatch_active_entities_query(client, config, args).await
            }
            AnalyticsCommand::Jobs { command } => match command {
                AnalyticsJobsCommand::Submit(args) => {
                    dispatch_async_job_submit(client, config, args).await
                }
                AnalyticsJobsCommand::Status(args) => {
                    dispatch_async_job_status(client, config, args).await
                }
                AnalyticsJobsCommand::Wait(args) => {
                    dispatch_async_job_wait(client, config, args).await
                }
                AnalyticsJobsCommand::Download(args) => {
                    dispatch_async_job_download(client, config, args).await
                }
            },
        },
        XCommand::Auth { .. } | XCommand::Config { .. } | XCommand::Doctor(_) => {
            unreachable!("auth/config/doctor are dispatched before loading X credentials")
        }
    }
}

fn x_result(
    client: &XClient,
    response: XResponse,
    endpoint: &str,
    account_id: Option<String>,
    warnings: Vec<String>,
) -> CommandResult {
    let mut envelope = OutputEnvelope::new(
        response.data,
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: endpoint.to_string(),
            object_id: account_id,
            request_id: response.request_id,
            report_run_id: None,
        },
    );
    envelope.paging = response.paging;
    if !warnings.is_empty() {
        envelope.warnings = Some(warnings);
    }
    CommandResult {
        envelope,
        exit_code: 0,
    }
}

fn x_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(data, endpoint, exit_code, Some(X_DEFAULT_API_VERSION))
}

async fn list_account_resource(
    client: &XClient,
    config: &XResolvedConfig,
    explicit_account_id: Option<&str>,
    resource_path: &str,
    endpoint: &str,
    collection: &XCollectionArgs,
    joined_params: &[(&str, &Vec<String>)],
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, explicit_account_id)?;
    let mut params = collection_params(collection);
    for (key, values) in joined_params {
        push_joined_param(&mut params, key, values);
    }

    let response = account_scoped::list_resource(
        client,
        &account_id,
        resource_path,
        &params,
        collection.pagination.all,
        collection.pagination.max_items,
    )
    .await?;
    Ok(x_result(
        client,
        response,
        endpoint,
        Some(account_id),
        vec![],
    ))
}

async fn get_account_resource(
    client: &XClient,
    config: &XResolvedConfig,
    explicit_account_id: Option<&str>,
    resource_path: &str,
    resource_id: &str,
    endpoint: &str,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, explicit_account_id)?;
    let response =
        account_scoped::get_resource(client, &account_id, resource_path, resource_id, &[]).await?;
    Ok(x_result(
        client,
        response,
        endpoint,
        Some(account_id),
        vec![],
    ))
}

async fn dispatch_sync_analytics(
    client: &XClient,
    config: &XResolvedConfig,
    args: AnalyticsQueryArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    validate_analytics_times(&args.start_time, &args.end_time, 7, "synchronous analytics")?;
    require_entity_ids(&args.entity_ids, 20, "entity IDs")?;
    require_values(&args.metric_groups, "metric groups")?;

    let response = x_analytics::query_sync(
        client,
        x_analytics::SyncAnalyticsQuery {
            account_id: &account_id,
            entity: args.entity.as_api_value(),
            entity_ids: &args.entity_ids,
            start_time: &args.start_time,
            end_time: &args.end_time,
            granularity: args.granularity.as_api_value(),
            placement: args.placement.as_api_value(),
            metric_groups: &args.metric_groups,
            country: args.country.as_deref(),
            platform: args.platform.as_deref(),
        },
    )
    .await?;

    Ok(x_result(
        client,
        response,
        "/x/analytics/query",
        Some(account_id),
        vec![],
    ))
}

async fn dispatch_reach_query(
    client: &XClient,
    config: &XResolvedConfig,
    args: ReachQueryArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    validate_whole_hour_time(&args.start_time, "start time")?;
    validate_whole_hour_time(&args.end_time, "end time")?;
    require_entity_ids(&args.ids, 20, "reach IDs")?;

    let response = x_analytics::query_reach(
        client,
        x_analytics::ReachQuery {
            account_id: &account_id,
            level: args.level.path(),
            ids: &args.ids,
            start_time: &args.start_time,
            end_time: &args.end_time,
        },
    )
    .await?;

    Ok(x_result(
        client,
        response,
        "/x/analytics/reach",
        Some(account_id),
        vec![],
    ))
}

async fn dispatch_active_entities_query(
    client: &XClient,
    config: &XResolvedConfig,
    args: ActiveEntitiesArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    validate_analytics_times(&args.start_time, &args.end_time, 90, "active entities")?;
    ensure_exclusive_filters(
        &args.campaign_ids,
        &args.funding_instrument_ids,
        &args.line_item_ids,
    )?;

    let response = x_analytics::query_active_entities(
        client,
        x_analytics::ActiveEntitiesQuery {
            account_id: &account_id,
            entity: args.entity.as_api_value(),
            start_time: &args.start_time,
            end_time: &args.end_time,
            campaign_ids: &args.campaign_ids,
            funding_instrument_ids: &args.funding_instrument_ids,
            line_item_ids: &args.line_item_ids,
        },
    )
    .await?;

    Ok(x_result(
        client,
        response,
        "/x/analytics/active-entities",
        Some(account_id),
        vec![],
    ))
}

async fn dispatch_async_job_submit(
    client: &XClient,
    config: &XResolvedConfig,
    args: AnalyticsJobSubmitArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    validate_async_job_range(
        &args.start_time,
        &args.end_time,
        args.segmentation_type.as_deref(),
    )?;
    require_entity_ids(&args.entity_ids, 20, "entity IDs")?;
    require_values(&args.metric_groups, "metric groups")?;
    validate_single_segmentation(args.segmentation_type.as_deref())?;

    let response = x_analytics::submit_job(
        client,
        x_analytics::AsyncJobQuery {
            account_id: &account_id,
            entity: args.entity.as_api_value(),
            entity_ids: &args.entity_ids,
            start_time: &args.start_time,
            end_time: &args.end_time,
            granularity: args.granularity.as_api_value(),
            placement: args.placement.as_api_value(),
            metric_groups: &args.metric_groups,
            segmentation_type: args.segmentation_type.as_deref(),
            country: args.country.as_deref(),
            platform: args.platform.as_deref(),
        },
    )
    .await?;

    Ok(x_result(
        client,
        response,
        "/x/analytics/jobs/submit",
        Some(account_id),
        vec![],
    ))
}

async fn dispatch_async_job_status(
    client: &XClient,
    config: &XResolvedConfig,
    args: AnalyticsJobStatusArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    let response = x_analytics::get_jobs(
        client,
        &account_id,
        &args.job_ids,
        args.pagination.cursor.as_deref(),
        args.pagination.page_size,
        args.pagination.all,
        args.pagination.max_items,
    )
    .await?;

    Ok(x_result(
        client,
        response,
        "/x/analytics/jobs/status",
        Some(account_id),
        vec![],
    ))
}

async fn dispatch_async_job_wait(
    client: &XClient,
    config: &XResolvedConfig,
    args: AnalyticsJobWaitArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    let job = wait_for_job(
        client,
        &account_id,
        &args.job_id,
        args.poll_interval_seconds,
        args.timeout_seconds,
    )
    .await?;

    Ok(x_command_result(
        json!({
            "provider": "x",
            "account_id": account_id,
            "job": job,
        }),
        "/x/analytics/jobs/wait",
        0,
    ))
}

async fn dispatch_async_job_download(
    client: &XClient,
    config: &XResolvedConfig,
    args: AnalyticsJobDownloadArgs,
) -> Result<CommandResult, XError> {
    let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    let job = if args.wait {
        wait_for_job(
            client,
            &account_id,
            &args.job_id,
            args.poll_interval_seconds,
            args.timeout_seconds,
        )
        .await?
    } else {
        get_single_job(client, &account_id, &args.job_id).await?
    };

    let status = job_status(&job).unwrap_or("unknown");
    if !job_is_success(status) {
        return Err(XError::InvalidArgument(format!(
            "job {} is not ready to download (status: {})",
            args.job_id, status
        )));
    }

    let download_url = job_download_url(&job).ok_or_else(|| {
        XError::InvalidArgument(format!(
            "job {} completed without a download URL in the response payload",
            args.job_id
        ))
    })?;

    let payload = client.download_json_url(download_url).await?;
    Ok(x_result(
        client,
        XResponse {
            data: payload,
            paging: None,
            request_id: None,
        },
        "/x/analytics/jobs/download",
        Some(account_id),
        vec![],
    ))
}

async fn get_single_job(client: &XClient, account_id: &str, job_id: &str) -> Result<Value, XError> {
    let response = x_analytics::get_jobs(
        client,
        account_id,
        &[job_id.to_string()],
        None,
        Some(1),
        false,
        Some(1),
    )
    .await?;

    response
        .data
        .as_array()
        .and_then(|jobs| jobs.first())
        .cloned()
        .ok_or_else(|| XError::Config(format!("job {} was not returned by the X Ads API", job_id)))
}

async fn wait_for_job(
    client: &XClient,
    account_id: &str,
    job_id: &str,
    poll_interval_seconds: u64,
    timeout_seconds: u64,
) -> Result<Value, XError> {
    let started_at = Instant::now();

    loop {
        let job = get_single_job(client, account_id, job_id).await?;
        let status = job_status(&job).unwrap_or("unknown");
        if job_is_success(status) {
            return Ok(job);
        }
        if job_is_failure(status) {
            return Err(XError::Config(format!(
                "job {} reached failure state: {}",
                job_id, status
            )));
        }
        if started_at.elapsed() >= Duration::from_secs(timeout_seconds) {
            return Err(XError::Config(format!(
                "timed out waiting for job {} after {} seconds",
                job_id, timeout_seconds
            )));
        }
        sleep(Duration::from_secs(poll_interval_seconds)).await;
    }
}

fn resolve_account_id(
    config: &XResolvedConfig,
    explicit_account_id: Option<&str>,
) -> Result<String, XError> {
    explicit_account_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| config.default_account_id.clone())
        .ok_or_else(|| {
            XError::InvalidArgument(
                "account ID is required. Pass --account-id or set providers.x.default_account_id in config.".to_string(),
            )
        })
}

fn collection_params(args: &XCollectionArgs) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(cursor) = args.pagination.cursor.as_deref() {
        params.push(("cursor".to_string(), cursor.to_string()));
    }
    if let Some(page_size) = args.pagination.page_size {
        params.push(("count".to_string(), page_size.to_string()));
    }
    if let Some(sort_by) = args.sort_by.as_deref() {
        params.push(("sort_by".to_string(), sort_by.to_string()));
    }
    params
}

fn push_joined_param(params: &mut Vec<(String, String)>, key: &str, values: &[String]) {
    if !values.is_empty() {
        params.push((key.to_string(), values.join(",")));
    }
}

fn required_credentials_present(auth: &XAuthSnapshot) -> bool {
    auth.consumer_key.present
        && auth.consumer_secret.present
        && auth.access_token.present
        && auth.access_token_secret.present
}

fn credential_store_check_ok(snapshot: &XConfigSnapshot) -> bool {
    snapshot.auth.credential_store_available
        || snapshot.auth.consumer_key.source == XSecretSource::ShellEnv
        || snapshot.auth.consumer_secret.source == XSecretSource::ShellEnv
        || snapshot.auth.access_token.source == XSecretSource::ShellEnv
        || snapshot.auth.access_token_secret.source == XSecretSource::ShellEnv
}

fn credential_store_detail(snapshot: &XConfigSnapshot) -> String {
    match snapshot.auth.credential_store_error.as_deref() {
        Some(error)
            if snapshot.auth.consumer_key.source == XSecretSource::ShellEnv
                || snapshot.auth.consumer_secret.source == XSecretSource::ShellEnv
                || snapshot.auth.access_token.source == XSecretSource::ShellEnv
                || snapshot.auth.access_token_secret.source == XSecretSource::ShellEnv =>
        {
            format!("shell env override active; OS credential store unavailable: {error}")
        }
        Some(error) => format!("OS credential store unavailable: {error}"),
        None if snapshot.auth.consumer_key.keychain_present
            || snapshot.auth.consumer_secret.keychain_present
            || snapshot.auth.access_token.keychain_present
            || snapshot.auth.access_token_secret.keychain_present =>
        {
            "stored X Ads credentials found in the OS credential store".to_string()
        }
        None if snapshot.auth.credential_store_available => {
            "OS credential store is available; no stored X Ads credentials found".to_string()
        }
        None => "OS credential store is unavailable".to_string(),
    }
}

fn secret_detail(env_var: &str, label: &str, status: &XSecretStatus) -> String {
    match status.source {
        XSecretSource::ShellEnv if status.keychain_present => {
            format!("{env_var} is set in shell env and overrides the stored {label}")
        }
        XSecretSource::ShellEnv => format!("{env_var} is set in shell env"),
        XSecretSource::Keychain => {
            format!("using stored X Ads {label} from the OS credential store")
        }
        XSecretSource::Missing => format!("{env_var} is missing"),
    }
}

fn x_auth_status_payload(auth: XAuthSnapshot) -> Value {
    json!({
        "provider": "x",
        "credential_store_available": auth.credential_store_available,
        "credential_store_error": auth.credential_store_error,
        "credentials": {
            "consumer_key": {
                "env_var": X_ADS_CONSUMER_KEY_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.consumer_key.present,
                "source": auth.consumer_key.source,
                "keychain_present": auth.consumer_key.keychain_present,
            },
            "consumer_secret": {
                "env_var": X_ADS_CONSUMER_SECRET_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.consumer_secret.present,
                "source": auth.consumer_secret.source,
                "keychain_present": auth.consumer_secret.keychain_present,
            },
            "access_token": {
                "env_var": X_ADS_ACCESS_TOKEN_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.access_token.present,
                "source": auth.access_token.source,
                "keychain_present": auth.access_token.keychain_present,
            },
            "access_token_secret": {
                "env_var": X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.access_token_secret.present,
                "source": auth.access_token_secret.source,
                "keychain_present": auth.access_token_secret.keychain_present,
            }
        }
    })
}

pub(crate) fn auth_storage_error(action: &str, error: &impl std::fmt::Display) -> XError {
    XError::Config(format!(
        "failed to {action} in the OS credential store: {error}{}",
        linux_secure_storage_hint()
    ))
}

fn linux_secure_storage_hint() -> &'static str {
    if cfg!(target_os = "linux") {
        " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet."
    } else {
        ""
    }
}

pub(crate) fn resolve_auth_inputs(args: &AuthSetArgs) -> Result<XAuthInputs, XError> {
    if args.stdin {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        return parse_auth_inputs_from_stdin(&input);
    }

    let consumer_key = normalize_nonempty(
        args.consumer_key
            .clone()
            .unwrap_or_else(|| prompt_password("X Ads consumer key: ").unwrap_or_default()),
        "consumer key",
    )?;
    let consumer_secret = normalize_nonempty(
        args.consumer_secret
            .clone()
            .unwrap_or_else(|| prompt_password("X Ads consumer secret: ").unwrap_or_default()),
        "consumer secret",
    )?;
    let access_token = normalize_nonempty(
        args.access_token
            .clone()
            .unwrap_or_else(|| prompt_password("X Ads access token: ").unwrap_or_default()),
        "access token",
    )?;
    let access_token_secret = normalize_nonempty(
        args.access_token_secret
            .clone()
            .unwrap_or_else(|| prompt_password("X Ads access token secret: ").unwrap_or_default()),
        "access token secret",
    )?;

    Ok(XAuthInputs {
        consumer_key,
        consumer_secret,
        access_token,
        access_token_secret,
    })
}

fn parse_auth_inputs_from_stdin(input: &str) -> Result<XAuthInputs, XError> {
    let values = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if values.len() != 4 {
        return Err(XError::InvalidArgument(
            "expected four non-empty lines on stdin: consumer key, consumer secret, access token, access token secret".to_string(),
        ));
    }

    Ok(XAuthInputs {
        consumer_key: normalize_nonempty(values[0].clone(), "consumer key")?,
        consumer_secret: normalize_nonempty(values[1].clone(), "consumer secret")?,
        access_token: normalize_nonempty(values[2].clone(), "access token")?,
        access_token_secret: normalize_nonempty(values[3].clone(), "access token secret")?,
    })
}

fn normalize_nonempty(value: String, label: &str) -> Result<String, XError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(XError::InvalidArgument(format!("{label} is required")));
    }
    Ok(trimmed.to_string())
}

fn parse_rfc3339(value: &str, label: &str) -> Result<OffsetDateTime, XError> {
    OffsetDateTime::parse(value.trim(), &Rfc3339)
        .map_err(|error| XError::InvalidArgument(format!("invalid {label}: {error}")))
}

fn validate_whole_hour_time(value: &str, label: &str) -> Result<OffsetDateTime, XError> {
    let timestamp = parse_rfc3339(value, label)?;
    if timestamp.minute() != 0 || timestamp.second() != 0 || timestamp.nanosecond() != 0 {
        return Err(XError::InvalidArgument(format!(
            "{label} must be aligned to a whole hour"
        )));
    }
    Ok(timestamp)
}

fn validate_analytics_times(
    start_time: &str,
    end_time: &str,
    max_days: i64,
    label: &str,
) -> Result<(), XError> {
    let start = validate_whole_hour_time(start_time, "start time")?;
    let end = validate_whole_hour_time(end_time, "end time")?;
    if end <= start {
        return Err(XError::InvalidArgument(
            "end time must be later than start time".to_string(),
        ));
    }
    let max_range = TimeDuration::days(max_days);
    if end - start > max_range {
        return Err(XError::InvalidArgument(format!(
            "{label} supports a maximum range of {max_days} days"
        )));
    }
    Ok(())
}

fn validate_async_job_range(
    start_time: &str,
    end_time: &str,
    segmentation_type: Option<&str>,
) -> Result<(), XError> {
    let limit_days = if segmentation_type.is_some() { 45 } else { 90 };
    validate_analytics_times(start_time, end_time, limit_days, "async analytics jobs")
}

fn require_entity_ids(values: &[String], max_count: usize, label: &str) -> Result<(), XError> {
    require_values(values, label)?;
    if values.len() > max_count {
        return Err(XError::InvalidArgument(format!(
            "{label} supports at most {max_count} values"
        )));
    }
    Ok(())
}

fn require_values(values: &[String], label: &str) -> Result<(), XError> {
    if values.is_empty() {
        return Err(XError::InvalidArgument(format!("{label} are required")));
    }
    Ok(())
}

fn validate_single_segmentation(segmentation_type: Option<&str>) -> Result<(), XError> {
    if let Some(segmentation_type) = segmentation_type {
        let segments = segmentation_type
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .count();
        if segments > 1 {
            return Err(XError::InvalidArgument(
                "X Ads async analytics supports only a single segmentation type".to_string(),
            ));
        }
    }
    Ok(())
}

fn ensure_exclusive_filters(
    campaign_ids: &[String],
    funding_instrument_ids: &[String],
    line_item_ids: &[String],
) -> Result<(), XError> {
    let populated = [
        !campaign_ids.is_empty(),
        !funding_instrument_ids.is_empty(),
        !line_item_ids.is_empty(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();

    if populated > 1 {
        return Err(XError::InvalidArgument(
            "active-entities accepts at most one of --campaign-id, --funding-instrument-id, or --line-item-id".to_string(),
        ));
    }
    Ok(())
}

fn job_status(job: &Value) -> Option<&str> {
    job.get("status")
        .and_then(Value::as_str)
        .or_else(|| job.get("state").and_then(Value::as_str))
        .or_else(|| job.get("job_status").and_then(Value::as_str))
}

fn job_download_url(job: &Value) -> Option<&str> {
    job.get("url")
        .and_then(Value::as_str)
        .or_else(|| job.get("download_url").and_then(Value::as_str))
        .or_else(|| job.get("location").and_then(Value::as_str))
}

fn job_is_success(status: &str) -> bool {
    matches!(
        status.to_ascii_uppercase().as_str(),
        "SUCCESS" | "SUCCEEDED" | "COMPLETED" | "DONE"
    )
}

fn job_is_failure(status: &str) -> bool {
    matches!(
        status.to_ascii_uppercase().as_str(),
        "FAILED" | "FAILURE" | "ERROR" | "CANCELLED" | "CANCELED"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_exclusive_filters, parse_auth_inputs_from_stdin, validate_analytics_times,
        validate_async_job_range,
    };

    #[test]
    fn parses_stdin_auth_inputs() {
        let inputs = parse_auth_inputs_from_stdin(
            " consumer-key \n consumer-secret \n access-token \n access-token-secret \n",
        )
        .unwrap();

        assert_eq!(inputs.consumer_key, "consumer-key");
        assert_eq!(inputs.consumer_secret, "consumer-secret");
        assert_eq!(inputs.access_token, "access-token");
        assert_eq!(inputs.access_token_secret, "access-token-secret");
    }

    #[test]
    fn rejects_misaligned_sync_window() {
        let error = validate_analytics_times(
            "2026-03-01T00:30:00Z",
            "2026-03-02T00:00:00Z",
            7,
            "synchronous analytics",
        )
        .unwrap_err();

        assert!(error.to_string().contains("whole hour"));
    }

    #[test]
    fn rejects_segmented_range_over_45_days() {
        let error = validate_async_job_range(
            "2026-01-01T00:00:00Z",
            "2026-02-20T00:00:00Z",
            Some("LOCATION"),
        )
        .unwrap_err();

        assert!(error.to_string().contains("45"));
    }

    #[test]
    fn active_entities_filters_are_exclusive() {
        let error = ensure_exclusive_filters(&["1".to_string()], &["2".to_string()], &Vec::new())
            .unwrap_err();

        assert!(error.to_string().contains("at most one"));
    }
}
