//! ome：Oh My Env 本机 Windows 环境部署管理 CLI（自 ohmypwsh ohmyenv.ps1 剥离的 Rust 实现）。
//! 第一阶段核心链路：catalog 读写、版本解析、下载缓存、sha256 校验，对外暴露 query/pin 两个子命令。

pub mod catalog;
pub mod checksum;
pub mod doctor;
pub mod download;
pub mod envpath;
pub mod extract;
pub mod install;
pub mod omerr;
pub mod package;
pub mod platform;
pub mod render;
pub mod resolve;
pub mod selfdeploy;
pub mod selfupdate;
pub mod status;
pub mod toolver;
pub mod verify;
pub mod vsbuild;
