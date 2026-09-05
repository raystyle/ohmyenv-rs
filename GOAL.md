# GOAL：任务目标管理

> 角色：**工作任务管理**，四个部分：**起点**（何时/为何发起）、**锚点**（当前锚定的目标 + 推进时间线）、
> **进程**（当前目标的进程）、**历史**（所有已完成目标的轨迹）。随工作实时更新。

## 起点

- **日期**：2026-09-05。
- **起点**：用户裁决命令原语重构第一批（PRD D07）：doctor 升核心命令做三层诊断（系统 / agent 二进制加版本加 token 可用性 / 依赖六类加环境错误检测），检测驱动安装、幂等检测安装；agent 二进制安装域由 oma 反转回 ome 承载（oma 收窄为配置 agent、hook、编排），追问链三轮六裁定稿。

## 锚点

> 当前锚定的目标 + 推进时间线：从起点到现在的关键推进节点（带日期），达成后整条移入「历史」轨迹。

- **锚定的目标**：D07 doctor 核心化与 agent 入册（三层诊断 + catalog agent 条目 + install/deploy 幂等覆盖 + oma 迁册配合）。

### 推进时间线

> 倒序。

| 日期 | 进展 |
| --- | --- |
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

- 当前目标：D07 doctor 核心化与 agent 入册。方案切片见 `PLAN.md`，进度见 `TODO.md`；完整迁移目标 M0..M4 已收盘（self update 五端自服务后二进制下发链退役），余量 M6（ohmypwsh 部署链退役配合）挂队列。

## 历史

> 所有已完成目标的轨迹，按日期倒序。

| 日期 | 目标 | 结果 |
| --- | --- | --- |
| 2026-08-31 | ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管 | 终结：被完整迁移裁决取代（2026-09-01）；对齐待办关闭为 superseded，数据迁移改走 P0026 M0 单向回流，mac 接管并入 M1 |
| 2026-08-31 | 扩展 ome 支持 Linux 本机软件部署（除 agent 外） | 达成：catalog 补齐 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl Linux 字段；新增 `tarxz-bin`、`linux_cdn_url`、UTF-16 校验清单、共享目录 flatten；query/install 闭环验证 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl；cargo test + `cargo check --target x86_64-pc-windows-gnu` + md 扫描全绿 |
| 2026-08-31 | WSL Linux 下接管 ome 开发 | 达成：WSL 侧 cargo build / cargo test 全绿（58 passed）；winreg 收进 target 依赖；selfdeploy/toolver/install 测试按平台 cfg 门控；R010 实测回填 |
| 2026-08-31 | ome 首版本机 Windows 域可用 | 达成：八命令落地，60 测试全绿；status 26/26、query 同 tag、daily 同判定全对齐 ohmyenv.ps1；self-deploy 幂等；推送 github.com/raystyle/ohmyenv |

## 维护规则

- **起点**：开工时写一句「何时发起 + 为什么发起」。
- **锚点**：推进中保持「锚定的目标 + 推进时间线」最新：每完成一个节点补一行（记日期 + 进展）。
- **进程**：只记当前目标的进行状态，达成后整条移入「历史」。
- **历史**：每个目标达成/变更时，记一条（日期 + 目标 + 结果），按日期倒序。
