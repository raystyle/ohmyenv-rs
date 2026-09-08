# GOAL：任务目标管理

> 角色：**工作任务管理**，四个部分：**起点**（何时/为何发起）、**锚点**（当前锚定的目标 + 推进时间线）、
> **进程**（当前目标的进程）、**历史**（所有已完成目标的轨迹）。随工作实时更新。

## 起点

- **日期**：2026-09-07。
- **起点**：用户裁定独立仓（PRD D18）：不再有 ohmypwsh 项目；ome 为独立仓库，只管命令在部署系统上功能完整。

## 锚点

> 当前锚定的目标 + 推进时间线：从起点到现在的关键推进节点（带日期），达成后整条移入「历史」轨迹。

- **锚定的目标**：D21 已交付（ohmycloud#8 三面对齐派单）。D08 第二批已落地（ohmyenv-rs#7）。余量：ohmycloud#9 补 ome/dev 段后 self update 断源实测并关 #7。

### 推进时间线

> 倒序。

| 日期 | 进展 |
| --- | --- |
| 2026-09-08 | D08 第二批落地：rust / vsbuild latest 段边车锚回落（download_latest_with_sidecar，先边车后资产、取不到拒绝无校验下载）；selfupdate 边车权威统一至 download 层；OME_TEST_MIRROR 6/6 绿（zoxide linux 回归、ffmpeg HEAD 在位）；发现镜像缺 ome/dev 段（dev 通道断源兜底 404），派 ohmycloud#9；既有批次分四枚收口推送，catalog 单提对齐 GitHub |
| 2026-09-08 | 镜像 v2.4 对账：zoxide linux 与 ffmpeg 双平台入镜三资产 HEAD 200 实证；TODO 闸门行回写；回报五点落 Temp 供镜像方读取 |
| 2026-09-07 | D21：ohmycloud#8 对齐 ome 自身与在管软件的分发、更新、安装（姊妹 #6） |
| 2026-09-07 | zoxide/ffmpeg linux_sha256 回填（官方资产哈希=GitHub digest）；HEAD 404 后派 ohmycloud 补种 |
| 2026-09-07 | D20 镜像闸门：ohmycloud#5 通报三处出入已核实；OME_TEST_MIRROR 2/2 绿；ffmpeg 边车=pin |
| 2026-09-07 | D20 收口：种子清单走 ISSUE；R014 加 seed-inventory.py；已派 ohmycloud#5，本仓跟踪 #8 |
| 2026-09-07 | D19 收口：ohmycloud 定为资源分发基建兄弟仓；AGENTS/README/ROADMAP/SKILL 同步 |
| 2026-09-07 | D18 收口：活文档与真机测试脱离 ohmypwsh；本机 doctor/status/query/pin/verify/heal --dry-run/install/skill/--llms 可跑；cli 28、clippy、四件套绿 |
| 2026-09-07 | D18 立项：不再有 ohmypwsh；活文档与真机测试脱离其对照；成功标准为本机命令面完整 |
| 2026-09-07 | D17 收口：去掉 package 子命令与 src/package.rs；cli 28、clippy 绿 |
| 2026-09-07 | D17 立项并收口：去掉 package 子命令与 src/package.rs |
| 2026-09-07 | D16 收口：删除 daily 子命令与日常更新编排；升级走 ome update；库测 87、cli 25、clippy 绿 |
| 2026-09-07 | D16 立项：用户裁定 daily 语义过重，升级一律 ome update |
| 2026-09-07 | D15 收口：去掉 deploy/download 子命令；install 一次完成下载、PATH、注册表与配置；update 为 install 到最新；库测 90、cli 25、clippy 绿 |
| 2026-09-07 | D15 终裁：去掉 deploy，install 含下载加 PATH/注册表/配置；此前「更名为 download」作废 |
| 2026-09-07 | D15 立项：用户裁定原语 install 改为 download（落盘不改 PATH，与 deploy 分工） |
| 2026-09-07 | D14 收口：catalog ffmpeg 入册；query 9.0.1 essentials zip；Windows install 幂等到 `D:\ohmyenv\ffmpeg\bin`（ffmpeg/ffprobe/ffplay），status locked=installed=9.0.1 path=true；mac ARM 空态 |
| 2026-09-07 | D14 立项：用户点名 github.com/FFmpeg/FFmpeg 与 ffmpeg.org/download.html；官方明示只提供源码，预编译 Windows gyan.dev、Linux BtbN、macOS evermeet（Intel only） |
| 2026-09-07 | 整仓审查缺陷收口：pin sha 必须同 tag；install --latest 不再污染旧 pin sha；下载 `.part` 提交；CDN `--version` 不沿用旧 tag；self update 镜像按通道分、替换部署位、catalog 同步保留 pin；doctor probe-fail/死链前缀/EnvRoot rustup/D13 总超时/verdict；localbin16 去 vault；Unix PATH 多目录；mac 只认 `mac_exe`；`--llms` 与缺子命令冒烟。PRD D09/D12/D13 已交付 |
| 2026-09-07 | mac 分发测试三平台闭环：lan-mac 断源双测绿（新增 darwin 资产用例 rmux macos-aarch64，e18c1d5）；self update 升级 42 工具新态（十类含密钥安全管理组）；doctor 三层加配置健康 darwin 形态实跑（telemetry WARN 如实）；install 幂等二连；回执 ohmycloud#3（D36 验收面三平台全绿）；另发 ohmycloud#4（ome 自身二进制 latest 段补种，self update 镜像兜底前提：mac 直连 GitHub DNS 超时实证）；vault 移除与 age/sops 独立密钥类（十类 42 工具，用户三裁）落码推送 |
| 2026-09-07 | D09 三件全收（3b5fc07 加本批）：根 SKILL.md（agent 发现入口，三原语口径）、`ome --llms`（全局 flag 紧凑命令清单，与 SKILL 同源；子命令改可选、缺子命令 agent 友好错误）、CTA（doctor 缺口建议 install、status 漂移建议 update 且 agent 类排除，走 stderr 不动冻结数据面）；真机 HINT 与 --llms 实测可见。D10 功能原语定档（三原语加派生，SKILL/R013/AGENTS 同步）。D11 立项（doctor 配置健康面：运行时/编译器/依赖配置判据从 heal-map 提取） |
| 2026-09-07 | D07 切片 3 收口：四家 agent force 沙盒真装布局全中（kimi 根级 kimi.exe 悬项实证、codex 嵌套 bin、claude/grok 直中）；update 与 daily 对 PATH 在位 agent 纳管跳过（与 install 同口径，真机四家全拦）；verify 无 agent 专属维度定案；R004 登记 OME_TEST_MIRROR 闸门；WSL status 自愈复验绿（zoxide/sheldon installed 回写，dotfiles 批余量闭环）。D07 四切片全收（切片 4 oma 侧已由并行会话交付，ohmypwsh#9 口径已 ome） |
| 2026-09-07 | D08 渠道链落地闭环：ohmycloud 种子终态 69/69（两条笔误资产补入后三方一致）后，ome 侧 download_asset_with_mirror 落码（官方失败回落 env.ohmygh.com，仅当有 sha 锚即 pin/官方 sums，双链错误信息）；幂等检测安装回归双绿（agent PATH 位与 EnvRoot 版本一致均 skip 不受镜像链影响）；断官方源闸门测试（OME_TEST_MIRROR，官方不可达回落且 sha 与 pin 一致）Windows 与 WSL 双端过；INDEX/README 同步，R004 闸门待登记 |
| 2026-09-07 | dotfiles 参考吸收批（S004）：zoxide 与 sheldon 入册（43 工具；用户三裁「只加 zoxide、sheldon 也加、其余学配置方式」）；zoxide Windows 本机 install 幂等加 sha 回填加 deploy 绿、WSL 双二进制就位（旧 ome 验版本步待新 dev 资产自愈）；下载链四段模式进 D08 设计、镜像对账一致（rsproxy 与 goproxy）；toolver 补两家正则 |
| 2026-09-05 | 范围口径裁定：Windows 本机加 WSL 双端先行，lan 三端以后待验收；镜像资产同口径先 win-x64 加 linux-x64 双平台（darwin 后补）；PLAN 完成定义与 TODO 切片同步，ohmycloud D36 行同步 |
| 2026-09-05 | D08 立项（分发兜底渠道，插队切片 3 前）：用户方向 Cloudflare Worker 加 S3 加自有域名自管分发，默认渠道切自建做准备；三裁 env.ohmygh.com 子域、41 工具全量一步到位、先手动种子后 Actions 自动；ohmycloud D36 同轮登记（基建），D07 切片 3 渠道部分并入统一实现 |
| 2026-09-05 | D07 切片 2 收口：doctor 三层成型（系统 os/arch/avx、agent 五字段健康块、依赖九类分组统计加 check 节十项）；token 探测边界两裁定落码（不取设置、不读环境变量防隐私；grok auth.json 与 kimi access_token 实证判据、claude/codex 无判据如实 na）；三态采集一次复用（run_doctor_with_status）；kv/json 双格式块化；85 加 26 测试全绿 |
| 2026-09-05 | D07 切片 1 收口：catalog 四家 agent 入册（claude/codex/grok/kimi，pin 与 sha 迁 oma 实证值，节序打头）；taxonomy 九类（加智能体依赖与运行时管理器依赖，uv/fnm 归位）；install 前探 PATH 在位即跳过（find_on_path 存量纳管）；status 对 agent 探测位换 PATH 首个命中（真机四行 installed/path/exe 全真）；toolver 补四家版本参数与正则；83 加 26 测试全绿，R001/INDEX/AGENTS/README 同步 |
| 2026-09-05 | D07 立项：追问链三轮六裁（ome 承载、oma agents install 迁册 ome、六类、先 doctor 命令面另立项、oma 登录态等四类检查归 agents 域、不取设置）；oma D06 当日五端闭环成果转为过渡态；前目标完整迁移余量 M6 挂队列（被动等 ohmypwsh 配合） |
| 2026-09-05 | 文档体系对齐 project-evo 重建（PRD D06，用户两裁：全面重整 + 全量追溯）：补 PRD（D01..D06 追溯登记）、docs\proven 归档节、ROADMAP 四态、AGENTS 文档义务表并瘦身 12633B 到 8432B；INDEX 以磁盘为唯一事实源对账（补 3 src 模块、6 tests 文件、diary 漏篇、.tools 漏件、断号注记、断链）；diary 禁字存量清剿 107 处（含标题括号 2 处），evo check 13 项全绿、四件套全绿 |
| 2026-09-02 | 写作规范转换收官：《中英文 Markdown 技术文档写作规范》v1.0 全文转项目规范（G001 v2 四类禁字符硬禁令、豁免区、替代写法），mdcharlint.py 进门禁成四件套，非归档区存量 105 处清剿，四门禁全绿；跨仓 ISSUE 矩阵就位（ohmypwsh#6#7、ohmyagents#2#3、ohmycloud#1，规范统一与 oma/omcf 集成条件）；reasonix 裁决不入册 |
| 2026-09-02 | 七类 taxonomy 定稿（操作编排/运行时/编译器/运行时衍生/多路复用/远程服务/命令工具依赖，统一「依赖」后缀，节序即类序，37 工具含 ome 自管条目）；status 七组头、黄金五件重生成；openssh 接管（MSI 型，本机 10.0p2 无缝幂等、resolve pin 版本权威加固）；doctor 两类误报修正（uv-git sha 豁免、派生资产白名单）加本机清理（死链 gopath/bin、孤儿 492MB），doctor 9 项全 OK |
| 2026-09-02 | issue 驱动批：bun mac 字段族补齐（lan-mac verify 收 7/7，issue #2 关闭）、query sha256 契约冻结与 package all 批量容错（issue #4）、CI 真网测试限流修复（GH_TOKEN 注入）后五端 self update 闭环（240B01E6/2C20A6F7/2D65822B） |
| 2026-09-02 | rust 接管建模收官（rustup.rs：rsproxy evergreen、EnvRoot 重定位、stable 滚动、cargo sparse 镜像；dev-rust heal 转原生，heal 部署域键全原生化）；M6 配合 ISSUE 通报 ohmypwsh |
| 2026-09-02 | M4 heal 移植收官：heal-map 42 键迁嵌入注册表四类归宿（install 原生、keys/mirror 原生移植、agent 休眠、外域路由）加 `ome heal [dim|all] [--dry-run]`；双平台验收闭环（Windows aria2/bunfig/dsKey、WSL go/dsKey 破坏-自愈-verify PASS、heal all 两连零 diff）；dev-rust 缺口（rustup 建模）另列待办 |
| 2026-09-02 | self update 三通道（dev 滚动 / --stable 正式版 / --git 源码）与 CI 双通道路由（main 出 dev 滚动源、v* tag 封版出正式 release）；接管 WSL msi（UTF-16 探测解码 + gsudo msiexec 提权）与 Docker Engine（set-docker.ps1 完整迁移：服务注册 + daemon.json + compose 插件 + 机器 PATH）；CI unix 测试三连修（fixture 平台布局、golden 双 oracle、HOME 归一）；oma init 脚手架落仓 |
| 2026-09-02 | Agent 友好 IO 重构（S003）：gh/git clone/incurs 源码三家实证，全局 `--format kv\|json\|jsonl` + `--json` 简写三格式渲染层、结构化错误 stderr 单行 JSON、verify/doctor 富字段（kv 兼容零破坏）、字段序稳定（preserve_order）、usage 跨平台去 .exe；四远端验收（WSL/lan-linux/lan-win/lan-mac 走 ssh） |
| 2026-09-02 | 定位定调（用户，三次澄清终版）：自适应承担**本地本系统**的工具与运行时环境部署、管理、诊断（平台自适配）；远程执行与集成归 ohmypwsh（本地调 ome、远程下发 ome 后 ssh 执行、密钥管理）；agent 四件套本体归 D:\ohmyagents；self-deploy 改名 init（兼容别名）、verify 流式输出、新增 doctor 部署异常诊断（首跑即抓出 PATH 死链与 850MB 缓存孤儿） |
| 2026-09-01 | M2 lan-win 收尾（sync-ome-lanwin.ps1 固化下发，远端 verify 9 项全 PASS 并修 gh 漂移）加 M3 第二批（ohmypwsh 三端部署域维度接线 ome verify，四端 73 项全绿，ps1 收敛为编排加密钥域） |
| 2026-09-01 | M3 第一批：ome verify 部署域验收命令（维度注册表，catalog 三态驱动）：Windows 9 维、WSL/lan-linux 各 7 维三端全 PASS，与 ps1 双跑部署域子集对账一致；「WSL 探测缺口」经平台严格化自解 |
| 2026-09-01 | 远端域批次（M2）：WSL 原生构建自部署 + lan-linux 二进制下发；install all 实证 WSL 31 项、lan-linux 24 项全幂等；修 all 容错、平台严格语义（cdn_url/platform_managed/repo effective）、shellcheck 探测三连 |
| 2026-09-01 | go/zig Windows 建模补齐（cdn 直链 + zip-dir 型，28/31 完美接管）与遥测关闭（pwsh 三重 + dotnet 运行时变量，set_user_env_var 新能力挂 install 双路径） |
| 2026-09-01 | 软件清单幂等接管 review（除 agent）：26/31 完美、5 项语义可解释；修 gh 2.98.0 布局变更致展平误判（M102 M004，gh 升 2.98.0）；新增 hold 版本锁定（bun 1.3.14 用户裁决）；登记 go/zig Windows 安装建模缺口 |
| 2026-09-01 | Windows 回接首轮：mac 10 笔拉回全门禁绿（校验器确认与 psd1 一致）；修真仓回归（仅 linux 字段工具在 Windows status 全挂），落 platform_managed 平台不适用容忍语义（空态行/跳过/拒绝），M106 记 M003 |
| 2026-09-01 | mac 批次全部收官并交接：开发接力回 Windows（D:\ohmyenv-rs）做 ohmypwsh 联动验收（P0026 M6 口径）；R011 一.4 刷新回接要点（基线门禁、平台 pin 不可覆盖、校验器只读），10 笔提交推送远端 |
| 2026-09-01 | mac 完美收敛：目录型运行时布局实证回填（pwsh/rmux/zig/go 四型）+ zip-bin/dir 提取型 + extra_bins 多二进制 + cdn pattern 平台族 + go/zig version 子命令与无前缀 tag 修复；starship 真装、vault/uv 升级、sops/go pin 补齐；终态 18 工具三态全等、其余如实空态；全门禁绿 |
| 2026-09-01 | M0 收官：psd1 Pos 侧一次性回流（linux/mac 静态族 + 平台 pin 分列 19+3 工具、mac pin 16 工具）、pin runtime 按平台分列（访问器/回写/sha 基准/状态判定）、转换器改只校验不再生（冲突报错增补放行）、go/zig/shellcheck 补录（kimi 按 agent 裁决剔除，31 工具）、R001 主权与同步纪律定案；校验器与全门禁绿 |
| 2026-09-01 | mac 首开发批次收官：M1 mac 字段族（`mac_*` + 三分支回退链 + exe 双语义）、jq/fnm mac 补录、R011 六项真机验证全绿、ohmypwsh 已部署 26 工具幂等接管实证（jq skipped 零重装零改写）；真机门禁修三坑（exe 双拼、clippy cfg 门控、测试沙盒漏 catalog 记 M102/M002） |
| 2026-09-01 | 交接 mac：用户赴 mac 接管开发，R011 刷新（新仓址、安装形态、M0 依赖 ohmypwsh psd1）；两仓推送远端 |
| 2026-09-01 | vsbuild 接管：evergreen 引导器（无 pin）+ gsudo 提权 + 机器级 PATH + MSBuild 稳定探测；真机幂等空转验证过（status installed=17.14.51 path=true） |
| 2026-09-01 | 立项批次：新址 clone 基线全绿（修 CRLF 检出敏感测试）、安装形态整改（用户程序目录自部署 + 数据目录 catalog 同步 + 旧 PATH 残留清理）、旧 clone 清理、R012 降级标注、三原语切换 |
| 2026-09-01 | 立项：完整迁移裁决；仓库改名 raystyle/ohmyenv-rs，本地 `D:\ohmyenv-rs` |

## 进程

> 当前目标的进程：只记录当前这一个目标的进行状态。

- 当前目标：D21 已交付。等 ohmycloud#8 裁决与 #7 补种通报。

## 历史

> 所有已完成目标的轨迹，按日期倒序。

| 日期 | 目标 | 结果 |
| --- | --- | --- |
| 2026-09-07 | D21 三面对齐派单 | 达成：ohmycloud#8 问分发/更新/安装如何对齐 ome 与在管软件 |
| 2026-09-07 | D20 种子清单 ISSUE 对齐 | 达成：R014 流程；ohmycloud#5 派单 85 对象差集 |
| 2026-09-07 | D19 ohmycloud 分发兄弟仓 | 达成：活文档写明 ohmycloud 是资源分发基建，ome 消费镜像不自建分发面 |
| 2026-09-07 | D18 独立仓 | 达成：不再依赖 ohmypwsh；本机命令面可跑 |
| 2026-09-07 | D17 去掉 package | 达成：删除子命令与 src/package.rs |
| 2026-09-07 | D16 去掉 daily | 达成：升级一律 ome update，不再做同主/跨主分流 |
| 2026-09-07 | D15 去掉 deploy | 达成：install 一次完成下载、PATH、注册表与配置；update 为 install 到最新 |
| 2026-09-07 | D14 ffmpeg 入册 | 达成：Windows GyanD essentials 9.0.1 真机闭环；Linux BtbN n9.0 已 pin；mac ARM 空态 |
| 2026-09-07 | 审查缺陷收口 | 达成：pin sha 同 tag、下载 .part、CDN 显式 version、doctor 判据、Unix PATH 多目录、mac 只认 mac_exe、文档四原语对齐 D09/D12/D13 |
| 2026-09-07 | D09 命令面 agent 友好化 | 达成：根 SKILL.md、`ome --llms`、CTA HINT、自适应 `ome skill` |
| 2026-09-07 | D07 doctor 核心化与 agent 入册 | 达成：四切片收口（catalog agent、三层 doctor、install 幂等与镜像、oma 迁册） |
| 2026-08-31 | ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管 | 终结：被完整迁移裁决取代（2026-09-01）；对齐待办关闭为 superseded，数据迁移改走 P0026 M0 单向回流，mac 接管并入 M1 |
| 2026-08-31 | 扩展 ome 支持 Linux 本机软件部署（除 agent 外） | 达成：catalog 补齐 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl Linux 字段；新增 `tarxz-bin`、`linux_cdn_url`、UTF-16 校验清单、共享目录 flatten；query/install 闭环验证 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl；cargo test + `cargo check --target x86_64-pc-windows-gnu` + md 扫描全绿 |
| 2026-08-31 | WSL Linux 下接管 ome 开发 | 达成：WSL 侧 cargo build / cargo test 全绿（58 passed）；winreg 收进 target 依赖；selfdeploy/toolver/install 测试按平台 cfg 门控；R010 实测回填 |
| 2026-08-31 | ome 首版本机 Windows 域可用 | 达成：八命令落地，60 测试全绿；status 26/26、query 同 tag、daily 同判定全对齐 ohmyenv.ps1；self-deploy 幂等；推送 github.com/raystyle/ohmyenv |

## 维护规则

- **起点**：开工时写一句「何时发起 + 为什么发起」。
- **锚点**：推进中保持「锚定的目标 + 推进时间线」最新：每完成一个节点补一行（记日期 + 进展）。
- **进程**：只记当前目标的进行状态，达成后整条移入「历史」。
- **历史**：每个目标达成/变更时，记一条（日期 + 目标 + 结果），按日期倒序。
