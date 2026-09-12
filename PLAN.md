# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D41 更名 Ark（用户 2026-09-11 定夺「ome 正式更名 Ark（Agent Runtime Kit）」；
> 2026-09-12 补裁：GitHub 仓改 raystyle/ark-rs、本地目录迁 `D:\ark-rs`、首发版 1.0.0）。

### 命名与兼容口径

| 面 | 定夺 |
| --- | --- |
| 产品名 | Ark，副题 Agent Runtime Kit |
| 命令名 | `ark` |
| 仓库名 | raystyle/ark-rs，GitHub 改名已生效 |
| 首发版 | 1.0.0，catalog、manifest、seq 门禁整体带入 |
| 术语 | 泊位 berth，即 EnvRoot 内安装位 |
| 环境变量 | `ARK_ROOT` 主名，读回 `OHMYENV_ROOT` 兼容 |
| 元数据目录 | XDG 与 `%LOCALAPPDATA%` 下 `ark` 主名，读回 `ohmyenv` 兼容 |
| EnvRoot 物理目录 | 不动，`D:\ohmyenv` 等原名保留 |
| 镜像段 | `ark/` 主名，兼容读 `ome/` |
| 自部署位 | ark 接管 ome 部署位，`ome` 别名过渡 |

### 前置约束

- 本地目录迁移（`D:\ohmyenv-rs` 改 `D:\ark-rs` 加 remote set-url）先于任何代码改动；会话与 herdr pane 于新路径重开。
- 本计划先过 herdr 飞轮对线（codex 评审），结论回执后才动手改码。
- 云端 `ark/` 段与 catalog 自管条目（`tools.ome`）更名属数据面与镜像面，依赖 omc 配合：herdr 知会在先，切换期双读不破供给。

### 方案骨架

1. **A 身份核心**：`Cargo.toml`（name 与 version 切 ark 与 1.0.0）；`src/main.rs`（clap name、LLMS_MANIFEST 命令表、用户面文案）；`src/platform.rs`（`metadata_dir` 与 `self_deploy_target` 切 ark 路径，旧 ohmyenv 与 ome 位读回识别）；环境变量族 ARK_* 主名、OME_* 与 OHMYENV_* 读回兼容（含 `OME_TEST_*` 测试闸门）。
2. **B 分发链**：`src/selfupdate.rs`（REPO 切 raystyle/ark-rs，镜像段 ark/dev 与 ark/stable，兼容读 ome/ 段）；`.github/workflows/build.yml`（mirror-r2 推 ark/ 段、资产名）；`.tools/seed.py`（段名参数化）。
3. **C 自举与存量兼容**：`src/catalog.rs`（CLOUD_CATALOG_KEY 切 `ark/catalog/`，切换期 ark/ 先、ome/ 回落，三重门不变）；profile 标记块写 `# >>> ark PATH`、读旧 ome 块（幂等不重复注册）；元数据目录迁移（catalog 副本、`.<名>.seq` 记录、`.last-sync` 标记自旧目录读回或搬迁）；`ark init` 落新部署位并纳管旧 ome 位（旧 PATH 条目清理、`ome` 别名过渡）；泊位语义零变化。
4. **D 文档与发版**：README、AGENTS、INDEX、SKILL、R001、R013 等全量更名，泊位术语入 R001；CHANGELOG 1.0.0 封版、ROADMAP 阶段状态；herdr 知会 omc（ark/ 段与自管条目）与 oma（命令面依赖确认）；tag v1.0.0 三平台 CI 绿后推。

### 自测面

1. 单测：`self_deploy_target` 与 `metadata_dir` 新路径断言；环境变量回退链（ARK_ROOT 与 OHMYENV_ROOT 双读）；profile 旧标记块识别；seq 门带入复验。
2. 集成：Windows 本机 `cargo test` 全量加 `cargo clippy` 干净；沙盒 install、query、status 双环境变量实证。
3. 自举：镜像 `ark/` 段三重门拉取（边车锚、解析、验签）；seq 升收降拒路径复验。
4. 自更新：REPO 改名后 self update 探测双态（ark/ 段与 ome/ 兼容读各实证一次）。
5. 三平台：CI matrix 全绿；cfg 门控文件三平台编译参与（M016 纪律）。
6. 兼容回归：旧 ome 部署位识别与接管；旧环境变量读回；旧 profile 块不重复注册；EnvRoot 原路径与存量工具零扰动。
7. 对线与文档门：A/B/C 实质改动推送前 codex review，回执入 diary；`rumdl check .` 加 `.tools` 三扫描绿。

### 完成定义

- `ark` 命令面与现 ome 功能等价（47 工具全链路）；旧名环境变量、旧部署位、旧 profile 块读回兼容不破。
- 云端 `ark/` 段与镜像锚经 omc 回执对账；v1.0.0 三平台 CI 绿、正式 release 与 ark/stable 直推、`self update` 部署位验收。
- 全量文档更名；正文 ome 残留仅限兼容口径与历史记录。

### 验收

- 本机 `ark status` 三态齐抽查、`ark catalog status` signature=valid、`ark self update --stable` 到 1.0.0。
- 存量机（WSL 或 lan 一端）升级实证：旧 ome 部署位被接管、旧环境变量名仍生效。
- cargo test 全绿加 clippy 干净加四件套绿；herdr 两侧回执入 diary。
