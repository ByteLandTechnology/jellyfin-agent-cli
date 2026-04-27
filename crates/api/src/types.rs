//! Common types used across the Jellyfin API.

use serde::{Deserialize, Serialize};

/// Unique identifier for a Jellyfin user
pub type UserId = String;

/// Unique identifier for a media item
pub type ItemId = String;

/// Unique identifier for a server
pub type ServerId = String;

/// Unique identifier for a session
pub type SessionId = String;

/// Authentication response from Jellyfin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Access token for authenticated requests
    #[serde(rename = "AccessToken")]
    pub access_token: Option<String>,

    /// User information
    #[serde(rename = "User")]
    pub user: Option<UserInfo>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID
    #[serde(rename = "Id")]
    pub id: UserId,

    /// Username
    #[serde(rename = "Name")]
    pub name: String,

    /// Server ID
    #[serde(rename = "ServerId")]
    pub server_id: Option<ServerId>,

    /// Whether this is the primary user
    #[serde(rename = "HasPassword")]
    pub has_password: bool,

    /// User policy
    #[serde(rename = "Policy")]
    pub policy: Option<UserPolicy>,
}

/// User policy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPolicy {
    /// Whether user is admin
    #[serde(rename = "IsAdministrator")]
    pub is_administrator: bool,

    /// Disabled features
    #[serde(rename = "DisabledChannels")]
    pub disabled_channels: Option<Vec<String>>,
}

/// Base item information (common to all media types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseItemDto {
    /// Item ID
    #[serde(rename = "Id")]
    pub id: ItemId,

    /// Item name
    #[serde(rename = "Name")]
    pub name: String,

    /// Item type
    #[serde(rename = "Type")]
    pub item_type: String,

    /// Media type (Audio, Video, Photo, etc.)
    #[serde(rename = "MediaType")]
    pub media_type: Option<String>,

    /// Sort name
    #[serde(rename = "SortName")]
    pub sort_name: Option<String>,

    /// Parent ID
    #[serde(rename = "ParentId")]
    pub parent_id: Option<ItemId>,

    /// Overview/description
    #[serde(rename = "Overview")]
    pub overview: Option<String>,

    /// Tagline
    #[serde(rename = "Tagline")]
    pub tagline: Option<String>,

    /// Genres
    #[serde(rename = "Genres")]
    pub genres: Option<Vec<String>>,

    /// Community rating
    #[serde(rename = "CommunityRating")]
    pub community_rating: Option<f64>,

    /// Critic rating
    #[serde(rename = "CriticRating")]
    pub critic_rating: Option<f64>,

    /// Index number (episode number, track number, etc.)
    #[serde(rename = "IndexNumber")]
    pub index_number: Option<u32>,

    /// Parent index number (season number, etc.)
    #[serde(rename = "ParentIndexNumber")]
    pub parent_index_number: Option<u32>,

    /// Production year
    #[serde(rename = "ProductionYear")]
    pub production_year: Option<u32>,

    /// Premiere date
    #[serde(rename = "PremiereDate")]
    pub premiere_date: Option<String>,

    /// End date
    #[serde(rename = "EndDate")]
    pub end_date: Option<String>,

    /// Run time in ticks
    #[serde(rename = "RunTimeTicks")]
    pub run_time_ticks: Option<u64>,

    /// Playback position in ticks
    #[serde(rename = "UserDataPlaybackPositionTicks")]
    pub playback_position_ticks: Option<u64>,

    /// Whether the item is played
    #[serde(rename = "UserDataPlayed")]
    pub played: Option<bool>,

    /// Whether the item is favorite
    #[serde(rename = "UserDataIsFavorite")]
    pub is_favorite: Option<bool>,

    /// Image tags
    #[serde(rename = "ImageTags")]
    pub image_tags: Option<std::collections::HashMap<String, String>>,

    /// Backdrop image tags
    #[serde(rename = "BackdropImageTags")]
    pub backdrop_image_tags: Option<Vec<String>>,

    /// Parent logo item ID
    #[serde(rename = "ParentLogoItemId")]
    pub parent_logo_item_id: Option<ItemId>,

    /// Parent backdrop item IDs
    #[serde(rename = "ParentBackdropItemId")]
    pub parent_backdrop_item_id: Option<ItemId>,

    /// Parent backdrop image tags
    #[serde(rename = "ParentBackdropImageTags")]
    pub parent_backdrop_image_tags: Option<Vec<String>>,

    /// Create date
    #[serde(rename = "DateCreated")]
    pub date_created: Option<String>,

    /// Last modified date
    #[serde(rename = "DateLastMediaAdded")]
    pub date_last_media_added: Option<String>,

    /// Channels
    #[serde(rename = "Channels")]
    pub channels: Option<u32>,

    /// Media streams
    #[serde(rename = "MediaSources")]
    pub media_sources: Option<Vec<MediaSource>>,

    /// People
    #[serde(rename = "People")]
    pub people: Option<Vec<BaseItemPerson>>,

    /// Studios
    #[serde(rename = "Studios")]
    pub studios: Option<Vec<NamePair>>,

    /// Tags
    #[serde(rename = "Tags")]
    pub tags: Option<Vec<String>>,
}

/// Person information (actors, directors, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseItemPerson {
    /// Person ID
    #[serde(rename = "Id")]
    pub id: Option<String>,

    /// Person name
    #[serde(rename = "Name")]
    pub name: String,

    /// Person type (Actor, Director, etc.)
    #[serde(rename = "Type")]
    pub person_type: Option<String>,

    /// Role
    #[serde(rename = "Role")]
    pub role: Option<String>,

    /// Sort order
    #[serde(rename = "SortOrder")]
    pub sort_order: Option<u32>,

    /// Image tag
    #[serde(rename = "PrimaryImageTag")]
    pub primary_image_tag: Option<String>,

    /// Base item ID for the person image
    #[serde(rename = "BaseItemId")]
    pub base_item_id: Option<String>,
}

/// Name ID pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamePair {
    /// Name
    #[serde(rename = "Name")]
    pub name: String,

    /// ID
    #[serde(rename = "Id")]
    pub id: Option<String>,
}

/// Media source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSource {
    /// Media type
    #[serde(rename = "Type")]
    pub media_type: Option<String>,

    /// Container
    #[serde(rename = "Container")]
    pub container: Option<String>,

    /// File size
    #[serde(rename = "Size")]
    pub size: Option<u64>,

    /// Name
    #[serde(rename = "Name")]
    pub name: Option<String>,

    /// Is remote
    #[serde(rename = "IsRemote")]
    pub is_remote: Option<bool>,

    /// Run time in ticks
    #[serde(rename = "RunTimeTicks")]
    pub run_time_ticks: Option<u64>,

    /// Supports direct stream
    #[serde(rename = "SupportsDirectStream")]
    pub supports_direct_stream: Option<bool>,

    /// Supports direct play
    #[serde(rename = "SupportsDirectPlay")]
    pub supports_direct_play: Option<bool>,

    /// Supports transcoding
    #[serde(rename = "SupportsTranscoding")]
    pub supports_transcoding: Option<bool>,

    /// Media streams
    #[serde(rename = "MediaStreams")]
    pub media_streams: Option<Vec<MediaStream>>,

    /// Supports probing
    #[serde(rename = "SupportsProbing")]
    pub supports_probing: Option<bool>,

    /// Video type (VideoFile, VideoIso, Dvd)
    #[serde(rename = "VideoType")]
    pub video_type: Option<String>,

    /// Iso type
    #[serde(rename = "IsoType")]
    pub iso_type: Option<String>,

    /// Path
    #[serde(rename = "Path")]
    pub path: Option<String>,

    /// Encoder path
    #[serde(rename = "EncoderPath")]
    pub encoder_path: Option<String>,

    /// Protocol
    #[serde(rename = "Protocol")]
    pub protocol: Option<String>,

    /// Genres
    #[serde(rename = "Genres")]
    pub genres: Option<Vec<String>>,

    /// Video stream
    #[serde(rename = "VideoStream")]
    pub video_stream: Option<MediaStream>,

    /// Audio streams
    #[serde(rename = "AudioStreams")]
    pub audio_streams: Option<Vec<MediaStream>>,

    /// Subtitle streams
    #[serde(rename = "SubtitleStreams")]
    pub subtitle_streams: Option<Vec<MediaStream>>,
}

/// Media stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStream {
    /// Stream codec
    #[serde(rename = "Codec")]
    pub codec: Option<String>,

    /// Stream type (Video, Audio, Subtitle)
    #[serde(rename = "Type")]
    pub stream_type: Option<String>,

    /// Stream index
    #[serde(rename = "Index")]
    pub index: Option<i32>,

    /// Language
    #[serde(rename = "Language")]
    pub language: Option<String>,

    /// Display title
    #[serde(rename = "DisplayTitle")]
    pub display_title: Option<String>,

    /// Is default
    #[serde(rename = "IsDefault")]
    pub is_default: Option<bool>,

    /// Is forced
    #[serde(rename = "IsForced")]
    pub is_forced: Option<bool>,

    /// Is external
    #[serde(rename = "IsExternal")]
    pub is_external: Option<bool>,

    /// Width
    #[serde(rename = "Width")]
    pub width: Option<u32>,

    /// Height
    #[serde(rename = "Height")]
    pub height: Option<u32>,

    /// Aspect ratio
    #[serde(rename = "AspectRatio")]
    pub aspect_ratio: Option<String>,

    /// Bit rate
    #[serde(rename = "BitRate")]
    pub bit_rate: Option<i32>,

    /// Bit depth
    #[serde(rename = "BitDepth")]
    pub bit_depth: Option<i32>,

    /// Sample rate
    #[serde(rename = "SampleRate")]
    pub sample_rate: Option<i32>,

    /// Channels
    #[serde(rename = "Channels")]
    pub channels: Option<i32>,

    /// Profile
    #[serde(rename = "Profile")]
    pub profile: Option<String>,

    /// Level
    #[serde(rename = "Level")]
    pub level: Option<f64>,

    /// Frame rate
    #[serde(rename = "RealFrameRate")]
    pub real_frame_rate: Option<f64>,

    /// Pixel format
    #[serde(rename = "PixelFormat")]
    pub pixel_format: Option<String>,

    /// Ref frames
    #[serde(rename = "RefFrames")]
    pub ref_frames: Option<i32>,

    /// NAL length size
    #[serde(rename = "NalLengthSize")]
    pub nal_length_size: Option<String>,

    /// Is AVCC
    #[serde(rename = "IsAVC")]
    pub is_avc: Option<bool>,

    /// Title
    #[serde(rename = "Title")]
    pub title: Option<String>,

    /// Codec tag
    #[serde(rename = "CodecTag")]
    pub codec_tag: Option<String>,

    /// Comment
    #[serde(rename = "Comment")]
    pub comment: Option<String>,

    /// Time base
    #[serde(rename = "TimeBase")]
    pub time_base: Option<String>,

    /// Codec time base
    #[serde(rename = "CodecTimeBase")]
    pub codec_time_base: Option<String>,

    /// Color space
    #[serde(rename = "ColorSpace")]
    pub color_space: Option<String>,

    /// Color transfer
    #[serde(rename = "ColorTransfer")]
    pub color_transfer: Option<String>,

    /// Color primaries
    #[serde(rename = "ColorPrimaries")]
    pub color_primaries: Option<String>,

    /// HDR format
    #[serde(rename = "HdrFormat")]
    pub hdr_format: Option<String>,

    /// Reference frames
    #[serde(rename = "ReferenceFrames")]
    pub reference_frames: Option<i32>,

    /// Delivery method
    #[serde(rename = "DeliveryMethod")]
    pub delivery_method: Option<String>,

    /// Delivery URL
    #[serde(rename = "DeliveryUrl")]
    pub delivery_url: Option<String>,

    /// Is interlaced
    #[serde(rename = "IsInterlaced")]
    pub is_interlaced: Option<bool>,

    /// Is anamorphic
    #[serde(rename = "IsAnamorphic")]
    pub is_anamorphic: Option<bool>,

    /// Channel layout
    #[serde(rename = "ChannelLayout")]
    pub channel_layout: Option<String>,

    /// Video range
    #[serde(rename = "VideoRange")]
    pub video_range: Option<String>,

    /// Video range type
    #[serde(rename = "VideoRangeType")]
    pub video_range_type: Option<String>,

    /// Display unit
    #[serde(rename = "DisplayUnit")]
    pub display_unit: Option<String>,

    /// External URL
    #[serde(rename = "ExternalUrl")]
    pub external_url: Option<String>,
}

/// Library item query options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemQuery {
    /// Parent ID
    #[serde(rename = "ParentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Include item types
    #[serde(rename = "IncludeItemTypes", skip_serializing_if = "Option::is_none")]
    pub include_item_types: Option<Vec<String>>,

    /// Exclude item types
    #[serde(rename = "ExcludeItemTypes", skip_serializing_if = "Option::is_none")]
    pub exclude_item_types: Option<Vec<String>>,

    /// Recursive
    #[serde(rename = "Recursive", skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,

    /// Sort by
    #[serde(rename = "SortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,

    /// Sort order
    #[serde(rename = "SortOrder", skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,

    /// Limit
    #[serde(rename = "Limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Start index
    #[serde(rename = "StartIndex", skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u32>,

    /// Search term
    #[serde(rename = "SearchTerm", skip_serializing_if = "Option::is_none")]
    pub search_term: Option<String>,

    /// Filters
    #[serde(rename = "Filters", skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<String>>,

    /// Genre IDs
    #[serde(rename = "GenreIds", skip_serializing_if = "Option::is_none")]
    pub genre_ids: Option<Vec<String>>,

    /// Studios
    #[serde(rename = "StudioIds", skip_serializing_if = "Option::is_none")]
    pub studio_ids: Option<Vec<String>>,

    /// Tags
    #[serde(rename = "Tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Years
    #[serde(rename = "Years", skip_serializing_if = "Option::is_none")]
    pub years: Option<Vec<u32>>,

    /// Image types
    #[serde(rename = "ImageTypes", skip_serializing_if = "Option::is_none")]
    pub image_types: Option<Vec<String>>,

    /// Is favorite
    #[serde(rename = "IsFavorite", skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,

    /// Is played
    #[serde(rename = "IsPlayed", skip_serializing_if = "Option::is_none")]
    pub is_played: Option<bool>,

    /// Is missing
    #[serde(rename = "IsMissing", skip_serializing_if = "Option::is_none")]
    pub is_missing: Option<bool>,

    /// Has subtitles
    #[serde(rename = "HasSubtitles", skip_serializing_if = "Option::is_none")]
    pub has_subtitles: Option<bool>,

    /// Has special feature
    #[serde(rename = "HasSpecialFeature", skip_serializing_if = "Option::is_none")]
    pub has_special_feature: Option<bool>,

    /// Has theme song
    #[serde(rename = "HasThemeSong", skip_serializing_if = "Option::is_none")]
    pub has_theme_song: Option<bool>,

    /// Has theme video
    #[serde(rename = "HasThemeVideo", skip_serializing_if = "Option::is_none")]
    pub has_theme_video: Option<bool>,

    /// Is place holder
    #[serde(rename = "IsPlaceHolder", skip_serializing_if = "Option::is_none")]
    pub is_place_holder: Option<bool>,

    /// Min date
    #[serde(rename = "MinDate", skip_serializing_if = "Option::is_none")]
    pub min_date: Option<String>,

    /// Max date
    #[serde(rename = "MaxDate", skip_serializing_if = "Option::is_none")]
    pub max_date: Option<String>,

    /// Min community rating
    #[serde(rename = "MinCommunityRating", skip_serializing_if = "Option::is_none")]
    pub min_community_rating: Option<f64>,

    /// Min critic rating
    #[serde(rename = "MinCriticRating", skip_serializing_if = "Option::is_none")]
    pub min_critic_rating: Option<f64>,

    /// Min premiere date
    #[serde(rename = "MinPremiereDate", skip_serializing_if = "Option::is_none")]
    pub min_premiere_date: Option<String>,

    /// Max premiere date
    #[serde(rename = "MaxPremiereDate", skip_serializing_if = "Option::is_none")]
    pub max_premiere_date: Option<String>,

    /// Enable image types
    #[serde(rename = "EnableImageTypes", skip_serializing_if = "Option::is_none")]
    pub enable_image_types: Option<Vec<String>>,

    /// Enable total record count
    #[serde(
        rename = "EnableTotalRecordCount",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_total_record_count: Option<bool>,

    /// Enable images
    #[serde(rename = "EnableImages", skip_serializing_if = "Option::is_none")]
    pub enable_images: Option<bool>,

    /// Exclude artist ids
    #[serde(rename = "ExcludeArtistIds", skip_serializing_if = "Option::is_none")]
    pub exclude_artist_ids: Option<Vec<String>>,

    /// Exclude item ids
    #[serde(rename = "ExcludeItemIds", skip_serializing_if = "Option::is_none")]
    pub exclude_item_ids: Option<Vec<String>>,

    /// Exclude location types
    #[serde(
        rename = "ExcludeLocationTypes",
        skip_serializing_if = "Option::is_none"
    )]
    pub exclude_location_types: Option<Vec<String>>,

    /// Include item ids
    #[serde(rename = "IncludeItemIds", skip_serializing_if = "Option::is_none")]
    pub include_item_ids: Option<Vec<String>>,

    /// Is 3 D
    #[serde(rename = "Is3D", skip_serializing_if = "Option::is_none")]
    pub is_3_d: Option<bool>,

    /// Is 4 K
    #[serde(rename = "Is4K", skip_serializing_if = "Option::is_none")]
    pub is_4_k: Option<bool>,

    /// Is HD
    #[serde(rename = "IsHD", skip_serializing_if = "Option::is_none")]
    pub is_hd: Option<bool>,

    /// Is SD
    #[serde(rename = "IsSD", skip_serializing_if = "Option::is_none")]
    pub is_sd: Option<bool>,

    /// Enable user data
    #[serde(rename = "EnableUserData", skip_serializing_if = "Option::is_none")]
    pub enable_user_data: Option<bool>,

    /// Image type limit
    #[serde(rename = "ImageTypeLimit", skip_serializing_if = "Option::is_none")]
    pub image_type_limit: Option<u32>,

    /// Min index number
    #[serde(rename = "MinIndexNumber", skip_serializing_if = "Option::is_none")]
    pub min_index_number: Option<u32>,

    /// Min parent index number
    #[serde(
        rename = "MinParentIndexNumber",
        skip_serializing_if = "Option::is_none"
    )]
    pub min_parent_index_number: Option<u32>,

    /// Max index number
    #[serde(rename = "MaxIndexNumber", skip_serializing_if = "Option::is_none")]
    pub max_index_number: Option<u32>,

    /// Max parent index number
    #[serde(
        rename = "MaxParentIndexNumber",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_parent_index_number: Option<u32>,

    /// Enable trailers
    #[serde(rename = "EnableTrailerTypes", skip_serializing_if = "Option::is_none")]
    pub enable_trailer_types: Option<Vec<String>>,

    /// Has overview
    #[serde(rename = "HasOverview", skip_serializing_if = "Option::is_none")]
    pub has_overview: Option<bool>,

    /// Group items into collections
    #[serde(
        rename = "GroupItemsIntoCollections",
        skip_serializing_if = "Option::is_none"
    )]
    pub group_items_into_collections: Option<bool>,

    /// Collapse box set items
    #[serde(
        rename = "CollapseBoxSetItems",
        skip_serializing_if = "Option::is_none"
    )]
    pub collapse_box_set_items: Option<bool>,

    /// Location types
    #[serde(rename = "LocationTypes", skip_serializing_if = "Option::is_none")]
    pub location_types: Option<Vec<String>>,
}

/// Item query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemQueryResult {
    /// Items
    #[serde(rename = "Items")]
    pub items: Vec<BaseItemDto>,

    /// Total record count
    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,

    /// Start index
    #[serde(rename = "StartIndex")]
    pub start_index: Option<u32>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    #[serde(rename = "ServerName")]
    pub server_name: String,

    /// Server version
    #[serde(rename = "Version")]
    pub version: String,

    /// Server ID
    #[serde(rename = "Id")]
    pub id: ServerId,

    /// Operating system
    #[serde(rename = "OperatingSystem")]
    pub operating_system: String,

    /// Architecture (SystemArchitecture in newer Jellyfin)
    #[serde(rename = "SystemArchitecture", default)]
    pub architecture: Option<String>,

    /// Product name
    #[serde(rename = "ProductName")]
    pub product_name: String,

    /// Local address
    #[serde(rename = "LocalAddress")]
    pub local_address: Option<String>,

    /// WAN address
    #[serde(rename = "WanAddress")]
    pub wan_address: Option<String>,

    /// Startup wizard completed
    #[serde(rename = "StartupWizardCompleted")]
    pub startup_wizard_completed: Option<bool>,
}

/// User list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResult {
    /// Users
    #[serde(rename = "Items")]
    pub items: Vec<UserInfo>,

    /// Total record count
    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,
}

/// Scheduled task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskInfo {
    /// Task ID
    #[serde(rename = "Id")]
    pub id: String,

    /// Task name
    #[serde(rename = "Name")]
    pub name: String,

    /// Task description
    #[serde(rename = "Description")]
    pub description: Option<String>,

    /// Task category
    #[serde(rename = "Category")]
    pub category: Option<String>,

    /// Whether the task is hidden
    #[serde(rename = "IsHidden")]
    pub is_hidden: bool,

    /// Current state (Idle, Running, etc.)
    #[serde(rename = "State")]
    pub state: String,

    /// Task key
    #[serde(rename = "Key")]
    pub key: Option<String>,

    /// Last execution result
    #[serde(
        rename = "LastExecutionResult",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_execution_result: Option<TaskExecutionResult>,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    /// Start time
    #[serde(rename = "StartTimeUtc")]
    pub start_time_utc: Option<String>,

    /// End time
    #[serde(rename = "EndTimeUtc")]
    pub end_time_utc: Option<String>,

    /// Status
    #[serde(rename = "Status")]
    pub status: Option<String>,
}

/// Device list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceListResult {
    /// Devices
    #[serde(rename = "Items")]
    pub items: Vec<DeviceInfo>,

    /// Total record count
    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,

    /// Start index
    #[serde(rename = "StartIndex")]
    pub start_index: Option<u32>,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device ID
    #[serde(rename = "Id")]
    pub id: String,

    /// Device name
    #[serde(rename = "Name")]
    pub name: String,

    /// Device type
    #[serde(rename = "Type")]
    pub device_type: Option<String>,

    /// Device manufacturer
    #[serde(rename = "Manufacturer")]
    pub manufacturer: Option<String>,

    /// Device model
    #[serde(rename = "Model")]
    pub model: Option<String>,

    /// Last user name
    #[serde(rename = "LastUserName")]
    pub last_user_name: Option<String>,

    /// App name
    #[serde(rename = "AppName")]
    pub app_name: Option<String>,

    /// App version
    #[serde(rename = "AppVersion")]
    pub app_version: Option<String>,

    /// Last user ID
    #[serde(rename = "LastUserId")]
    pub last_user_id: Option<String>,

    /// Last seen date
    #[serde(rename = "DateLastActivity")]
    pub date_last_activity: Option<String>,

    /// Device capabilities
    #[serde(rename = "Capabilities", skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<DeviceCapabilities>,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Playable media types
    #[serde(rename = "PlayableMediaTypes")]
    pub playable_media_types: Option<Vec<String>>,

    /// Supported commands
    #[serde(rename = "SupportedCommands")]
    pub supported_commands: Option<Vec<String>>,

    /// Supports media control
    #[serde(rename = "SupportsMediaControl")]
    pub supports_media_control: Option<bool>,

    /// Supports persistent identifier
    #[serde(rename = "SupportsPersistentIdentifier")]
    pub supports_persistent_identifier: Option<bool>,
}

/// Notification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    #[serde(rename = "Items")]
    pub items: Vec<NotificationDto>,

    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,
}

/// Notification DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDto {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "UserId")]
    pub user_id: Option<String>,

    #[serde(rename = "Title")]
    pub title: Option<String>,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "Date")]
    pub date: Option<String>,

    #[serde(rename = "IsRead")]
    pub is_read: Option<bool>,

    #[serde(rename = "Level")]
    pub level: Option<String>,
}

/// Plugin info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "Version")]
    pub version: Option<String>,

    #[serde(rename = "Status")]
    pub status: Option<String>,

    #[serde(rename = "ConfigurationUrl")]
    pub configuration_url: Option<String>,

    #[serde(rename = "CanUninstall")]
    pub can_uninstall: Option<bool>,

    #[serde(rename = "ImageUrl")]
    pub image_url: Option<String>,
}

/// Channel result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResult {
    #[serde(rename = "Items")]
    pub items: Vec<BaseItemDto>,

    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,
}

/// Channel item result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelItemResult {
    #[serde(rename = "Items")]
    pub items: Vec<BaseItemDto>,

    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,
}

/// Session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    #[serde(rename = "Id")]
    pub id: Option<String>,

    #[serde(rename = "UserId")]
    pub user_id: Option<String>,

    #[serde(rename = "UserName")]
    pub user_name: Option<String>,

    #[serde(rename = "DeviceId")]
    pub device_id: Option<String>,

    #[serde(rename = "DeviceName")]
    pub device_name: Option<String>,

    #[serde(rename = "ClientName")]
    pub client_name: Option<String>,

    #[serde(rename = "ClientVersion")]
    pub client_version: Option<String>,

    #[serde(rename = "LastActivityDate")]
    pub last_activity_date: Option<String>,

    #[serde(rename = "NowPlayingItem")]
    pub now_playing_item: Option<BaseItemDto>,

    #[serde(rename = "PlayState")]
    pub play_state: Option<PlayState>,
}

/// Playback state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayState {
    #[serde(rename = "PositionTicks")]
    pub position_ticks: Option<u64>,

    #[serde(rename = "CanSeek")]
    pub can_seek: Option<bool>,

    #[serde(rename = "IsPaused")]
    pub is_paused: Option<bool>,

    #[serde(rename = "IsMuted")]
    pub is_muted: Option<bool>,

    #[serde(rename = "VolumeLevel")]
    pub volume_level: Option<i32>,
}

/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    #[serde(rename = "Id")]
    pub id: u32,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "ShortOverview")]
    pub short_overview: Option<String>,

    #[serde(rename = "Overview")]
    pub overview: Option<String>,

    #[serde(rename = "Severity")]
    pub severity: Option<String>,

    #[serde(rename = "Date")]
    pub date: Option<String>,

    #[serde(rename = "UserId")]
    pub user_id: Option<String>,

    #[serde(rename = "UserPrimaryImageTag")]
    pub user_primary_image_tag: Option<String>,

    #[serde(rename = "Type")]
    pub entry_type: Option<String>,
}

/// Activity log result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogResult {
    #[serde(rename = "Items")]
    pub items: Vec<ActivityLogEntry>,

    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: Option<u32>,

    #[serde(rename = "StartIndex")]
    pub start_index: Option<u32>,
}

/// Remote search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchQuery {
    #[serde(rename = "SearchInfo")]
    pub search_info: RemoteSearchInfo,

    #[serde(rename = "ItemType")]
    pub item_type: Option<String>,

    #[serde(rename = "MetadataCountryCode")]
    pub metadata_country_code: Option<String>,

    #[serde(rename = "MetadataLanguageCode")]
    pub metadata_language_code: Option<String>,
}

/// Remote search info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchInfo {
    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Year")]
    pub year: Option<u32>,

    #[serde(rename = "ProviderIds")]
    pub provider_ids: Option<std::collections::HashMap<String, String>>,
}

/// Remote search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchResult {
    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "PremiereDate")]
    pub premiere_date: Option<String>,

    #[serde(rename = "ProductionYear")]
    pub production_year: Option<u32>,

    #[serde(rename = "ProviderIds")]
    pub provider_ids: Option<std::collections::HashMap<String, String>>,

    #[serde(rename = "SearchProviderName")]
    pub search_provider_name: Option<String>,

    #[serde(rename = "Overview")]
    pub overview: Option<String>,

    #[serde(rename = "InheritedFromParent")]
    pub inherited_from_parent: Option<String>,
}

/// Playlist info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistInfo {
    #[serde(rename = "Id")]
    pub id: Option<String>,

    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "OpenAccess")]
    pub open_access: Option<bool>,

    #[serde(rename = "Shares")]
    pub shares: Option<Vec<serde_json::Value>>,

    #[serde(rename = "ItemIds")]
    pub item_ids: Option<Vec<String>>,

    #[serde(rename = "Users")]
    pub users: Option<Vec<serde_json::Value>>,
}

/// Create playlist response (only contains ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylistResponse {
    #[serde(rename = "Id")]
    pub id: String,
}
