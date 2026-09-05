# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**：按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

承接 ohmypwsh 部署、验收、自愈完整迁移，按 ohmypwsh 仓库 P0026 方案的 M0..M6 里程碑推进（登记日 2026-09-01）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 切片 1：catalog agent 条目建模 | 已完成 | 四家入册（pin/sha 迁 oma 实证值；双渠道的 CDN 兜底与 kimi manifest 渠道留切片 3）；taxonomy 九类（用户裁定 uv 是 python 运行时管理兼运行时、fnm 纯 node 管理）；install agent 存量纳管跳过与 status PATH 探测同步落地；R001/INDEX/AGENTS/README 同步 | 2026-09-05 |
| 切片 2：doctor 三层重构 | 已完成 | 系统层（os/arch/avx 三指令集，oma caps 同口径）、agent 层（binary/version/locked/drift/token 五字段；token 只探凭据文件不取设置不读环境变量，grok/kimi 实证判据、claude/codex 无稳定判据如实 na）、依赖层（九类分组 tools/missing/drift）加现有十项归 check 节；三态采集一次复用；85 加 26 测试全绿 | 2026-09-05 |
| 切片 3：install/deploy 幂等覆盖 agent | 待办 | agent 条目接 install/status/pin/daily/verify；存量纳管语义；oma 自管根布局对齐策略 | |
| 切片 4：oma 迁册配合与跨仓收口 | 待办 | oma deprecated 标记、agents.toml 数据源注记、ohmypwsh#9 口径更新、五端验收 | |
| D07 立项登记 | 已完成 | 追问链三轮六裁；PRD D07、GOAL 起点锚点切换（完整迁移 M6 挂队列）、PLAN 四切片 | 2026-09-05 |
| M6 ohmypwsh 链退役配合 | 挂队列 | 被动等 ohmypwsh 配合（ohmypwsh#9 承接方口径随 D07 更新为 ome）；验收口径见 P0026 M6 | |
| 文档体系 evo 对齐重建（D06） | 已完成 | PRD 全量追溯（D01..D06）、proven 归档节、ROADMAP、AGENTS 义务表与瘦身、INDEX 磁盘对账重整、diary 禁字清剿；evo check 13 项全绿、四件套全绿 | 2026-09-05 |

### 已完成批次

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 写作规范转换 | 已完成 | 《中英文 Markdown 技术文档写作规范》v1.0 全文转项目规范（用户裁决）：G001 v2 四类禁字符硬禁令与豁免区、mdcharlint.py 进四件套门禁（默认豁免 diary/proven）、非归档区 105 处存量清剿、四门禁全绿；跨仓规范与集成 ISSUE 矩阵（ohmypwsh#7、ohmyagents#2#3、ohmycloud#1） | 2026-09-02 |
| 七类 taxonomy 定稿 | 已完成 | 37 工具归七类（操作编排/运行时/编译器/运行时衍生/多路复用/远程服务/命令工具依赖，用户四轮裁决定稿）：catalog 节序即类序、ome 自管条目（extract=ome-self，clap version）、status 七组头、fixture 与黄金五件重生成（linux WSL 真机核对）；oma/omcf 待兄弟仓集成、reasonix 裁决不入册 | 2026-09-02 |
| openssh 接管与 doctor 修正 | 已完成 | Windows OpenSSH MSI 型条目（本机 10.0p2 无缝幂等接管、ssh -V stderr 探测、resolve pin 版本权威加固）；doctor 两类误报修正（uv-git sha 豁免、派生资产白名单）与本机清理（PATH 死链 gopath/bin、真孤儿 492MB），doctor 终态 9 项全 OK | 2026-09-02 |
| issue 驱动批与五端闭环 | 已完成 | bun mac 字段族（lan-mac verify 7/7，#2 关闭）；query sha256 契约冻结与 package all 容错（#4）；CI 真网限流修复（resolve GH_TOKEN 注入与测试步环境）后五端 self update 闭环 | 2026-09-02 |
| rust 接管建模 | 已完成 | rustup 引导器型条目（rsproxy 直链 evergreen 无 pin、stable 滚动即更新）；rustup.rs 迁移 set-rust.ps1 全语义（四用户环境变量重定位 EnvRoot、init 引导、update stable 保最新、cargo sparse 镜像 config.toml、cargo bin 用户 PATH）；heal dev-rust 键转原生 install；query/pin/update/daily/doctor evergreen 分支齐接；真机幂等验收 | 2026-09-02 |
| M4 heal 移植 | 已完成 | heal-map.psd1 42 键迁嵌入注册表（44 行，toolRoot/aria2 平台分列）四类归宿：install 类原生安装（pin 驱动幂等，rust 建模后 16 行）、dsKey/akKey/bunfig/goproxy 原生移植（heal-keys/heal-mirror 逐字节语义）、agent 域 12 键休眠（裁决）、非 ome 域 8 行路由提示（secret-guard/残留/compileMatrix/POSIX aria2）；mac-* 4 键别名归一；`ome heal [dim|all] [--dry-run]`；验收双平台闭环（Windows aria2/bunfig/dsKey + WSL go/dsKey 破坏-自愈-verify PASS、heal all 两连零 diff） | 2026-09-02 |
| M3 第二批：verify-five-ends 接线 | 已完成 | ohmypwsh 三端部署域维度行改透传 ome verify（ome=FAIL 兜底、compileMatrix 引用 ome 结果）；复验四端 73 项全绿，ps1 收敛为编排加密钥域 | 2026-09-01 |
| lan-win 端 ome 下发 | 已完成 | 固化 sync-ome-lanwin.ps1（sha 对比按需传二进制加 catalog 每次同步）；远端 verify 9 项全 PASS（顺带修 gh 漂移），M2 四端齐 | 2026-09-01 |
| M3 第一批：ome verify | 已完成 | 部署域维度注册表（Windows 9 维、POSIX 7 维），catalog 三态加文件存在判定，`dim=PASS/FAIL` 收割行与 ps1 同构；三端全 PASS 与 ps1 双跑对账一致；「WSL 探测缺口」经平台严格化自解 | 2026-09-01 |
| M2 WSL 与 lan-linux 批次 | 已完成 | WSL 原生构建自部署加二进制下发（Sync sha 对比按需传），install all 实证 WSL 31 项、lan-linux 24 项全幂等；lan-win 收尾另列待办；M5 远端通道以「ohmypwsh 编排调系统 ssh」形态达成（agent 归属已按裁决移出） | 2026-09-01 |
| go/zig Windows 安装建模 | 已完成 | cdn 直链（go.dev/dl、ziglang.org/download）加 zip-dir 提取型（zig 版本目录不展平）；pin 四键含 sha 回填，install 幂等跳过；status 28/31 完美接管 | 2026-09-01 |
| 遥测关闭 | 已完成 | pwsh：msi 属性 DISABLE_TELEMETRY 之外补 POWERSHELL_TELEMETRY_OPTOUT=1 与 POWERSHELL_UPDATECHECK=Off；dotnet：DOTNET_CLI_TELEMETRY_OPTOUT=1；新能力 set_user_env_var（HKCU/profile 标记块）挂 install 双路径钩子，注册表三值实证 | 2026-09-01 |
| Windows 回接与基线门禁 | 已完成 | mac 10 笔拉回后全门禁绿（76 测试、fmt/clippy、md 三件套、校验器「托管 26 节与 psd1 一致」）；修 Windows 真仓回归：仅 linux 字段工具（shellcheck）无通用 exe 致 status 全挂，加 platform_managed 容忍（status 空态行、install/update/daily/pin/query 跳过、package 拒绝），M106 记 M003 | 2026-09-01 |
| M1 后续：mac 完美收敛 | 已完成 | 目录型运行时布局实证回填（pwsh 装入 powershell/7、rmux 装入 ~/.local 树、zig 装入 ~/zig 版本目录、go 装入 ~/.local/go，新增 zip-bin/targz-dir/tarxz-dir 提取型）；extra_bins 多二进制（age-keygen/sg/uvx 实证）；cdn pattern 平台族修 vault；go/zig version 子命令与无前缀 tag 修复；sops/uv/vault/go mac pin 补齐（starship 真装、vault/uv 升级接管）；终态 18 工具 locked=installed、pwsh 版本对齐、win-only 与 official 布局如实空态 | 2026-09-01 |
| M0 数据主权与回流 | 已完成 | psd1 Pos 侧一次性回流（19 在管工具 linux 静态族+pin、16 工具 mac pin，钉版本 pattern 解开为 [0-9.]+）；pin runtime 平台分列（访问器/回写/sha 基准/status/update 全链切平台键，黄金双 oracle）；import-catalog.ps1 改只校验不再生（冲突报错、ome 增补放行、平台族完整性）；go/zig/shellcheck 补录（kimi 按不管理 agent 裁决剔除，共 31 工具）；同步纪律定案入 R001 | 2026-09-01 |
| M1 mac 字段族与真机验证 | 已完成 | Tool 增 `mac_*` 族、`effective_*` 三分支回退链（mac、linux、通用）、exe 双语义修复；jq/fnm 首批 mac 补录；R011 六项真机验证全绿（70 测试、install/deploy 幂等接管在位 jq、self-deploy 部署态解析、package fnm、gnu 交叉 check）；真机门禁修三坑（exe 双拼、clippy cfg、测试沙盒漏 catalog 记 M102） | 2026-09-01 |
| vsbuild 接管 | 已完成 | evergreen 引导器（无 pin 无 sha）+ gsudo 自动提权 + 机器级 PATH + MSBuild 稳定探测；74 测试全绿、真机幂等空转验证；Windows SDK 仍留 ohmypwsh（ISO 分离） | 2026-09-01 |
| 新址 clone 与基线门禁 | 已完成 | D:\ohmyenv-rs 基线 993e77b；修 CRLF 检出敏感测试（夹具 CRLF 检出时 `\r\n` 双转）后门禁全绿 | 2026-09-01 |
| 安装形态整改 | 已完成 | 自部署进 `%LOCALAPPDATA%\Programs\ome`、catalog 同步 `%LOCALAPPDATA%\ohmyenv`、旧 PATH 残留清理；catalog 解析扩四级；68 测试全绿 | 2026-09-01 |
| 承接完整迁移登记 | 已完成 | R012 降级标注（被 P0026 取代）、三原语切换、INDEX 与 diary 登记 | 2026-09-01 |
| ohmypwsh catalog 与部署脚本对齐 | superseded | 被完整迁移裁决取代（2026-09-01）：数据改走 M0 单向回流，不再双向对齐 | 2026-09-01 |
