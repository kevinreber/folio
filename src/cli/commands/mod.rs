mod capture;
mod config_cmd;
mod daemon;
mod delete;
mod digest;
mod edit;
mod export_cmd;
mod list;
mod match_cmd;
mod promote;
mod search;
mod serve;
mod show;
mod stats;
mod sync;
mod tracking;
mod tui_cmd;
mod watch;

pub use capture::run as capture;
pub use config_cmd::{init as config_init, run as config};
pub use daemon::run_action as daemon;
pub use delete::run as delete;
pub use digest::{review, run as digest};
pub use edit::run as edit;
pub use export_cmd::{import, run as export};
pub use list::run as list;
pub use match_cmd::run as match_job;
pub use promote::run as promote;
pub use search::run as search;
pub use serve::{run_api as serve_api, run_mcp as serve_mcp};
pub use show::run as show;
pub use stats::run as stats;
pub use sync::run as sync;
pub use tracking::{
    import_calendar, import_meeting, import_transcript, screen_capture, track, voice,
};
pub use tui_cmd::run as tui;
pub use watch::run as watch;
