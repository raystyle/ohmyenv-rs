//! ome：Oh My Env 本机跨平台环境部署管理 CLI。
//! 三原语 doctor / install / status；派生 query、update、pin、verify、heal、
//! init、skill、self update。catalog 为唯一 pin 源。

pub mod catalog;
pub mod checksum;
pub mod docker;
pub mod doctor;
pub mod download;
pub mod envpath;
pub mod extract;
pub mod heal;
pub mod install;
pub mod omerr;
pub mod platform;
pub mod render;
pub mod resolve;
pub mod rustup;
pub mod selfdeploy;
pub mod selfupdate;
pub mod status;
pub mod toolver;
pub mod verify;
pub mod vsbuild;
