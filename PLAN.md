# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D41 更名 Ark（用户 2026-09-11 定夺「ome 正式更名 Ark（Agent Runtime Kit）」；
> 2026-09-12 补裁：GitHub 仓改 raystyle/ark-rs、本地目录迁 `D:\ark-rs`、首发版 1.0.0）。
> 已过 codex 对线两轮：首轮 16 条结论（R1 至 R16）与自测七缺口全采纳；复核轮五条残差收口；回执入 diary 2026-09-12。

### 命名与兼容口径

| 面 | 定夺 |
| --- | --- |
| 产品名 | Ark，副题 Agent Runtime Kit |
| 命令名 | `ark` |
| 仓库名 | raystyle/ark-rs，GitHub 改名已生效 |
| 首发版 | 1.0.0，catalog、manifest、seq 门禁整体带入 |
| 术语 | 泊位 berth，即 EnvRoot 内安装位 |
| 环境变量 | `ARK_ROOT` 主名，读回 `OHMYENV_ROOT` 兼容；同设时主名优先（进自测断言） |
| 元数据目录 | XDG 与 `%LOCALAPPDATA%` 下 `ark` 主名，读回 `ohmyenv` 兼容 |
| EnvRoot 物理目录 | 不动，`D:\ohmyenv` 等原名保留 |
| 镜像段 | `ark/` 主名；切换期 `ark/` 与 `ome/` 双写，读序 ark/ 先、ome/ 回落 |
| 自部署位 | ark 接管 ome 部署位；`ome` 别名 = 部署位同目录 exe 副本，init 与 self update 重建 |

### 前置约束

- 本地目录迁移已完成（`D:\ark-rs` 就位）；herdr 飞轮对线两轮（评审与复核）先于任何代码改动，结论回执入 diary 后动码。
- 云端 `ark/` 段铺设归 omc：**沿用现 seq 单调序列不重计数**，tools 与 manifest 同批同签同 seq（否则 D40 门拒收而非回落）。安全序：omc 先铺 `ark/`，端再切主名，过观察窗后 omc 停 `ome/`；**停 `ome/` 段与停 `ome-*` 资产名的判据同为存量机水位清零**（旧二进制的镜像回落与资产名硬编码都在 ome 面）。
- catalog 自管条目（`tools.ome`）更名属数据面由 omc 落；端引擎双接受 `ome-self` 与 `ark-self`（数据面改名可单方回退）。
- 签名密钥位裁定：私钥**保留** `~/.config/ome/catalog-signing.key` 不随名迁移（密钥材料零扰动，公钥内嵌不变）；R015 注明，后续轮换时再议。
- 迁移检查单：仓内 agent hook（`.claude/settings.json`、`.codex/hooks.json`、`.grok/hooks`）指向旧目录 `D:/ohmyenv-rs`，须 `oma init --pretrust` 重建加 shim 实体探活（同型坑：hook 在但 shim 缺失）。

### 方案骨架

1. **A 身份核心**：`Cargo.toml`（name 与 version 切 ark 与 1.0.0）加内部 crate 路径（`use ome::` 等）加测试 harness（`cargo_bin("ome")`、`OME_CATALOG` 沙盒注入）为同一批原子项；`src/main.rs`（clap name、LLMS_MANIFEST 命令表、用户面文案、`ome-self|ark-self` 双接受六分支）；`src/platform.rs`（`metadata_dir` 与 `self_deploy_target` 切 ark 路径，旧 ohmyenv 与 ome 位读回识别）；环境变量族 ARK_* 主名、OME_* 与 OHMYENV_* 读回兼容（含 `OME_TEST_*` 测试闸门，同设主名优先）；git 通道产物名按 `CARGO_PKG_NAME` 派生；内部临时名与 UA 顺手清（extract tmp、selfupdate UA、resolve UA、seed.py 标识）。
2. **B 分发链**：`src/selfupdate.rs`（REPO 切 raystyle/ark-rs；镜像段读序 ark/ 先、ome/ 回落）；`.github/workflows/build.yml`（mirror-r2 切换期 `ark/` 与 `ome/` **双写**：同一 job 对两段各跑一次 seed.py，保 `ome/` 段不断供；资产名 `ark-*` 主加 `ome-*` 兼容名同 release 双附，v1.0.0 起至少保留到存量机水位清零）；`.tools/seed.py`（段名参数化）；`src/doctor.rs` 三网络探针并入（改指新仓与 `ark/stable`，URL 与源码常量同源）。
3. **C 自举与存量兼容**：`src/catalog.rs`（CLOUD_CATALOG_KEY 切 `ark/catalog/`，切换期 ark/ 先、ome/ 回落，三重门不变）；profile 写入面按 `rg "# >>>"` 全枚举双读加旧块幂等清理（PATH 块、env 块、fnm 钩子块）加 `~/.config/ohmyenv-secrets` 载体与 rc 挂钩行；元数据目录搬迁七件套（`tools.toml`、`tools.toml.minisig`、`manifest.toml`、`manifest.toml.minisig`、`.tools.toml.seq`、`.manifest.toml.seq`、`.last-sync`），旧目录只读；**旧目录已见 seq 补记新目录：首刷一次，旧二进制并存期每次刷新前复核旧目录取 max（防并行期旧目录水位续升），旧二进制退役后旧目录冻结（触发判据与停段同源：存量机水位清零；复用 `record_seen_seq_from_local` 语义，防回滚门重置成 seen=0）**；`ark init` 落新部署位、`ome` 别名同目录 exe 副本、旧位清单逐项处置（`Programs\ome`、`~/.local/bin/ome`、`EnvRoot\ome\bin`，清旧 PATH 条目但别名保留可调）；泊位语义零变化。
4. **D 文档与发版**：README、AGENTS、INDEX、SKILL、R001、R013 等更名，泊位术语入 R001；文档清扫以 rg 残留门收口（`rg -n -i "ohmyenv|OME_|\bome\b"`，词边界防 home/some 误报；逐条裁定，仅兼容读与历史记录可留）；R013 冻结面登记更名 breaking 加 ome 别名过渡；R015 注明密钥位保留；CHANGELOG 1.0.0 封版、ROADMAP 阶段状态；herdr 知会 omc（ark/ 铺段续 seq、双写窗口、停段判据、自管条目更名、命令面契约 omc agent deploy 冻结调用 ome install 改 ark install 加别名兼容窗口）与 oma（命令面依赖确认）；tag v1.0.0 三平台 CI 绿后推。

### 自测面

1. 单测：`self_deploy_target` 与 `metadata_dir` 新路径断言；环境变量回退链（ARK_ROOT 与 OHMYENV_ROOT 双读、同设主名优先）；profile 全部旧标记块识别；**元数据迁移用例两断言：新目录视图仍拦旧 seq（防 seen=0 窗口）；旧目录水位高于新目录时复核取 max**。
2. 集成：Windows 本机 `cargo test` 全量加 `cargo clippy` 干净；沙盒 install、query、status 双环境变量实证。
3. 自举：镜像段三态矩阵（`ark/` 命中；`ark/` 404 落 `ome/`；双 404 报错）；三重门（边车锚、解析、验签）；seq 升收降拒路径复验。
4. 自更新：REPO 改名后 self update 探测双态（ark/ 段与 ome/ 兼容读各实证一次）；**旧二进制 0.4.2 真机跨版本升级到新仓 1.0.0 全链用例**；gh api 兜底路径对 301 的跟随实测。
5. 三平台：CI matrix 全绿；cfg 门控文件三平台编译参与（M016 纪律）。
6. 兼容回归：旧 ome 部署位识别与接管、旧位清单清理；旧环境变量读回；POSIX 旧 env 与 fnm 块、Windows 注册表旧条目读回；**旧 PATH 条目清理后 `ome` 别名仍可调（三平台各一条）**；EnvRoot 原路径与存量工具零扰动。
7. 对线与文档门：A/B/C 实质改动推送前 codex review，回执入 diary；`rumdl check .` 加 `.tools` 三扫描绿；**doctor 探针 URL 与源码常量同源机检**。

### 完成定义

- `ark` 命令面与现 ome 功能等价（47 工具全链路）；旧名环境变量、旧部署位、旧 profile 块读回兼容不破；`ome` 别名三平台可调。
- 云端 `ark/` 段与镜像锚经 omc 回执对账（seq 续号无回退）；v1.0.0 三平台 CI 绿、正式 release 双资产名、ark/stable 与 ome/stable 双写、`self update` 部署位验收（含 0.4.2 旧二进制升级实证）。
- 全量文档更名；正文 ome 残留仅限兼容口径与历史记录（rg 残留门留档）；存量机水位观测在案，停 ome/ 段动作待水位清零另行执行。

### 验收

- 本机 `ark status` 三态齐抽查、`ark catalog status` signature=valid、`ark self update --stable` 到 1.0.0。
- 存量机（WSL 或 lan 一端）升级实证：旧 ome 部署位被接管、旧环境变量名仍生效、`ome` 别名可调。
- cargo test 全绿加 clippy 干净加四件套绿；herdr 两侧回执入 diary。
