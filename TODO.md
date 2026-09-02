# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

承接 ohmypwsh 部署、验收、自愈完整迁移，按 ohmypwsh 仓库 P0026 方案的 M0..M6 里程碑推进（登记日 2026-09-01）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| rust 接管建模（dev-rust heal 转原生） | 待办 | rustup + stable + rsproxy 建模（set-rust.ps1 迁移：用户环境变量重定位 RUSTUP_HOME/CARGO_HOME 到 EnvRoot、rustup-init 引导）；完成后 heal 注册表 dev-rust 键由路由转原生 install | |
| M6 ohmypwsh 链退役配合 | 待办 | 配合 ohmypwsh 逐域 deprecated，验收口径见 P0026 M6；mac 侧随时按 R011 一.4 复验 | |

### 已完成批次

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| M4 heal 移植 | 已完成 | heal-map.psd1 42 键迁嵌入注册表（44 行，toolRoot/aria2 平台分列）四类归宿：install 类 17 键原生安装（pin 驱动幂等）、dsKey/akKey/bunfig/goproxy 原生移植（heal-keys/heal-mirror 逐字节语义）、agent 域 12 键休眠（裁决）、非 ome 域 8 键路由提示（secret-guard/残留/compileMatrix/dev-rust/POSIX aria2）；mac-* 4 键别名归一；`ome heal [dim|all] [--dry-run]`；验收双平台闭环（Windows aria2/bunfig/dsKey + WSL go/dsKey 破坏-自愈-verify PASS、heal all 两连零 diff）；dev-rust 缺口另列待办 | 2026-09-02 |
| M3 第二批：verify-five-ends 接线 | 已完成 | ohmypwsh 三端部署域维度行改透传 ome verify（ome=FAIL 兜底、compileMatrix 引用 ome 结果）；复验四端 73 项全绿，ps1 收敛为编排加密钥域 | 2026-09-01 |
| lan-win 端 ome 下发 | 已完成 | 固化 sync-ome-lanwin.ps1（sha 对比按需传二进制加 catalog 每次同步）；远端 verify 9 项全 PASS（顺带修 gh 漂移），M2 四端齐 | 2026-09-01 |
| M3 第一批：ome verify | 已完成 | 部署域维度注册表（Windows 9 维、POSIX 7 维），catalog 三态加文件存在判定，`dim=PASS/FAIL` 收割行与 ps1 同构；三端全 PASS 与 ps1 双跑对账一致；「WSL 探测缺口」经平台严格化自解 | 2026-09-01 |
| M2 WSL 与 lan-linux 批次 | 已完成 | WSL 原生构建自部署加二进制下发（Sync sha 对比按需传），install all 实证 WSL 31 项、lan-linux 24 项全幂等；lan-win 收尾另列待办；M5 远端通道以「ohmypwsh 编排调系统 ssh」形态达成（agent 归属已按裁决移出） | 2026-09-01 |
| go/zig Windows 安装建模 | 已完成 | cdn 直链（go.dev/dl、ziglang.org/download）加 zip-dir 提取型（zig 版本目录不展平）；pin 四键含 sha 回填，install 幂等跳过；status 28/31 完美接管 | 2026-09-01 |
| 遥测关闭 | 已完成 | pwsh：msi 属性 DISABLE_TELEMETRY 之外补 POWERSHELL_TELEMETRY_OPTOUT=1 与 POWERSHELL_UPDATECHECK=Off；dotnet：DOTNET_CLI_TELEMETRY_OPTOUT=1；新能力 set_user_env_var（HKCU/profile 标记块）挂 install 双路径钩子，注册表三值实证 | 2026-09-01 |
| Windows 回接与基线门禁 | 已完成 | mac 10 笔拉回后全门禁绿（76 测试、fmt/clippy、md 三件套、校验器「托管 26 节与 psd1 一致」）；修 Windows 真仓回归：仅 linux 字段工具（shellcheck）无通用 exe 致 status 全挂，加 platform_managed 容忍（status 空态行、install/update/daily/pin/query 跳过、package 拒绝），M106 记 M003 | 2026-09-01 |
| M1 后续：mac 完美收敛 | 已完成 | 目录型运行时布局实证回填（pwsh→powershell/7、rmux→~/.local 树、zig→~/zig 版本目录、go→~/.local/go，新增 zip-bin/targz-dir/tarxz-dir 提取型）；extra_bins 多二进制（age-keygen/sg/uvx 实证）；cdn pattern 平台族修 vault；go/zig version 子命令与无前缀 tag 修复；sops/uv/vault/go mac pin 补齐（starship 真装、vault/uv 升级接管）；终态 18 工具 locked=installed、pwsh 版本对齐、win-only 与 official 布局如实空态 | 2026-09-01 |
| M0 数据主权与回流 | 已完成 | psd1 Pos 侧一次性回流（19 在管工具 linux 静态族+pin、16 工具 mac pin，钉版本 pattern 解开为 [0-9.]+）；pin runtime 平台分列（访问器/回写/sha 基准/status/update 全链切平台键，黄金双 oracle）；import-catalog.ps1 改只校验不再生（冲突报错、ome 增补放行、平台族完整性）；go/zig/shellcheck 补录（kimi 按不管理 agent 裁决剔除，共 31 工具）；同步纪律定案入 R001 | 2026-09-01 |
| M1 mac 字段族与真机验证 | 已完成 | Tool 增 `mac_*` 族、`effective_*` 三分支回退链（mac→linux→通用）、exe 双语义修复；jq/fnm 首批 mac 补录；R011 六项真机验证全绿（70 测试、install/deploy 幂等接管在位 jq、self-deploy 部署态解析、package fnm、gnu 交叉 check）；真机门禁修三坑（exe 双拼、clippy cfg、测试沙盒漏 catalog 记 M102） | 2026-09-01 |
| vsbuild 接管 | 已完成 | evergreen 引导器（无 pin 无 sha）+ gsudo 自动提权 + 机器级 PATH + MSBuild 稳定探测；74 测试全绿、真机幂等空转验证；Windows SDK 仍留 ohmypwsh（ISO 分离） | 2026-09-01 |
| 新址 clone 与基线门禁 | 已完成 | D:\ohmyenv-rs 基线 993e77b；修 CRLF 检出敏感测试（夹具 CRLF 检出时 `\r\n` 双转）后门禁全绿 | 2026-09-01 |
| 安装形态整改 | 已完成 | 自部署进 `%LOCALAPPDATA%\Programs\ome`、catalog 同步 `%LOCALAPPDATA%\ohmyenv`、旧 PATH 残留清理；catalog 解析扩四级；68 测试全绿 | 2026-09-01 |
| 承接完整迁移登记 | 已完成 | R012 降级标注（被 P0026 取代）、三原语切换、INDEX 与 diary 登记 | 2026-09-01 |
| ohmypwsh catalog 与部署脚本对齐 | superseded | 被完整迁移裁决取代（2026-09-01）：数据改走 M0 单向回流，不再双向对齐 | 2026-09-01 |
