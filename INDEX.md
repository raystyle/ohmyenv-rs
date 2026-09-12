# INDEX：唯一索引

> 全仓库唯一索引：文档编号表、目录结构、代码文件位置。查文档先搜本文件。

## 文档编号表

> 前缀：P=proven 方案归档 / S=research 研究 / R=references 开发参考 / G=guide 元规范 / M=mistakes 错误速查。

| 编号 | 文件 | 主题 |
| --- | --- | --- |
| G001 | `docs\guide\G001-文档标准细则-命名写作规范与rumdl检查.md` | 文档命名、写作规范与 rumdl 检查 |
| G002 | `docs\guide\G002-研究标准细则-结构与六态标记.md` | 研究文档结构与六态标记 |
| G003 | `docs\guide\G003-工作流标准细则-从登记到归档五步.md` | 想法从登记到归档五步工作流 |
| G004 | `docs\guide\G004-经验沉淀细则-成功与错误经验分治.md` | 经验沉淀分治：成功进 proven/references，错误进 mistakes，同型坑升格 |
| R001 | `docs\references\R001-catalog数据模式-tools-toml字段与pin语义.md` | `catalog\tools.toml` 字段模式与 pin 回写语义 |
| R004 | `docs\references\R004-测试标准细则-分层断言与门禁流程.md` | 测试分层断言与门禁（真机对齐闸门 OME_TEST_REAL） |
| R005 | `docs\references\R005-选型研究细则-cratesio与github双通道.md` | Rust 库与项目选型双通道 |
| R008 | `docs\references\R008-项目工具Python库选型细则-pypi与uv.md` | 项目工具 Python 选库与 uv |
| R009 | `docs\references\R009-项目工具PowerShell模块选型细则-psgallery与psresourceget.md` | 项目工具 PowerShell 模块选型 |
| R010 | `docs\references\R010-linux开发接管-环境准备与构建验证.md` | Linux 开发主机接管，已归档：工具链、构建验证、平台门控、两端分工 |
| R011 | `docs\references\R011-mac开发接管-环境准备与构建验证.md` | mac 开发主机接管：工具链、构建验证、mac 目录/PATH 策略、三端分工 |
| R012 | `docs\references\R012-ohmypwsh与ome对齐清单-linux-windows.md` | 历史对齐清单：已降级为 catalog 数据迁移参考（2026-09-01 被完整迁移裁决取代；D18 源项目不存在） |
| R013 | `docs\references\R013-Agent友好IO契约-输出格式退出码与冻结面.md` | 输出三格式、数据错误分流、命令数据块字段、退出码与对外冻结契约（自 README 收敛） |
| R014 | `docs\references\R014-ohmycloud种子清单-ISSUE派任务与对齐.md` | 与 ohmycloud 协调（herdr 通道）；agent 部署委托与管辖边界终版（D29/D37：清单数据权威归 omc）；版本对齐水位 |
| R016 | `docs\references\R016-云端清单与manifest标准.md` | 云端 catalog 与 manifest 数据标准草案（两件分离同签、schema 版本化、三层 DSL 原语、跨仓分工 omc 维护 ome 执行；D39） |
| R015 | `docs\references\R015-软件清单发布更新与播种标准.md` | 清单发布、更新与软件播种三流程唯一标准（三重门、三通道、双路线、管辖边界两域分治；D35/D36） |
| S001 | `docs\research\S001-incurs选型研究-不迁移只吸收三模式.md` | incurs 框架选型裁决：不迁移，吸收错误结构、单一渲染层、帮助元数据三模式 |
| S002 | `docs\research\S002-command-line-rust方法论-测试oracle与输出纪律.md` | Command-Line Rust 全书方法论研究：测试 oracle 三件套值得吸收，错误/参数形态 ome 已超越 |
| S003 | `docs\research\S003-Agent友好IO研究-gh与git与incurs代码实证.md` | gh 与 git clone 与 incurs 源码三家 Agent 友好 IO 实证：吸收三格式渲染、结构化错误、字段序稳定；过滤/分页/自描述不吸收 |
| S005 | `docs\research\S005-GitHub发版与分支合并流程调研-三模型与自动化.md` | GitHub 发版与分支合并标准流程：三模型（GitHub Flow / Git Flow / Trunk-Based）、合并三式、分支保护、发版载体与自动化工具实证；阶段七封版决策输入 |
| S004 | `docs\research\S004-dotfiles吸收研究-下载链模式与配置栈.md` | dotfiles 吸收：zoxide/sheldon 入册、下载链四段模式进 D08、镜像对账一致、zsh/starship/sheldon 配置栈记档 |
| S007 | `docs\research\S007-安装清单DSL研究-winget与scoop实证.md` | 安装清单 DSL 选型：winget 纯声明与 scoop 任意脚本的实证对照、ome 能力对位与缺口、三层提案（L1 声明原语/L2 受控命令/L3 不进）与信任模型（D39） |
| S006 | `docs\research\S006-云端清单校验研究-相关项目签名与防篡改实现对照.md` | 云端软件清单防 MITM 的实现对照：SOPS 与 age 的真实模型（加密非签名）、apt 与 RPM 与 Helm 与 HashiCorp 的分离签名、Sparkle 的公钥内嵌、TUF 与 PEP 458 的回滚防护、npm 与 Sigstore 的同源信任；对 ome 的候选与建议（D34） |

编号注记：R002/R003、R006/R007 号段属 ohmyagents 产品特定（命令细则、rmux、agent 信任），2026-08-31 文档体系自 ohmyagents 平移时留空不复用；P0001 起待首个方案达成后启用（见下方方案归档节）。

不编号文档：`docs\guide\template.md`（方案模板）。

## 方案归档

> 位置 `docs\proven\`；PNNNN 四位编号（P0001 起，接最大号不复用），文件名即标题，从 `docs\guide\template.md` 起步。归档时机：目标验收全绿后回填（AGENTS 文档义务表）；封存即冻结，过时只在正文顶部加注记不改写历史；TODO 残表清退留指针指向本节。分界见 G004。

| 编号 | 文件 | 主题 | 归档日 |
| --- | --- | --- | --- |

首个归档条件：D18 独立仓命令面本机闭环验收后，D05 迁移全程（M0..M4，M6 取消）一并归档为 P0001。

## 根目录文档

| 文件 | 角色 |
| --- | --- |
| `PRD.md` | 需求清单管理（四原语之首，D 编号与生命周期） |
| `GOAL.md` | 任务目标管理（起点/锚点/进程/历史） |
| `PLAN.md` | 当前目标规划指导 |
| `TODO.md` | 当前目标任务进度清单 |
| `AGENTS.md` | 协作规则最高约束（含文档义务表） |
| `README.md` | 项目简介与快速开始 |
| `SKILL.md` | agent 发现入口（何时用 ome、命令图与三原语口径、镜像与幂等语义；D09） |
| `CHANGELOG.md` | 版本里程碑 |
| `ROADMAP.md` | 阶段与里程碑（四态） |

## 目录结构

| 目录 | 说明 |
| --- | --- |
| `src\` | Rust 源码，平铺模块（无子目录） |
| `catalog\` | 已退役（D37 完全解耦：清单数据权威迁 ohmycloud catalog-seed，权威件已删，消费走云端三件套；`tests\fixtures\tools.toml` 为测试夹具；格式契约 R001） |
| `tests\` | 集成测试（逐文件职责见下节代码文件位置） |
| `.tools\` | 可复用脚本归档（清单见 `.tools\README.md`：import-catalog.ps1、seed.py、catalog-sign、inject-guide-d25.py、inject-probe-d28.py、seed-inventory.py、md-ref-scan.py、md-heading-scan.py、mdcharlint.py、md-replace.py、md-ref-allow.txt） |
| `docs\` | proven/research/references/guide/mistakes/diary 六类 |
| `bin\` | init 产物（self-deploy 兼容别名）（ome.exe，注册进用户 PATH；git 忽略） |

## 项目日记

> 位置 `docs\diary\`；一天一篇，当天总结与自省。

| 日期 | 文件 | 主题 |
| --- | --- | --- |
| 2026-08-31 | `docs\diary\2026-08-31-ome立项-文档体系与结构平移.md` | 立项、脚手架、catalog 转换、文档体系平移 |
| 2026-08-31 | `docs\diary\2026-08-31-ohmypwsh与omelinux-windows对齐先行.md` | ohmypwsh 与 ome 的 Linux/Windows 对齐先行 |
| 2026-08-31 | `docs\diary\2026-08-31-ome扩展linux本机部署支持.md` | 平台抽象层 platform.rs、Linux 本机部署支持 |
| 2026-08-31 | `docs\diary\2026-08-31-开发主机切换到mac.md` | 开发主机由 WSL 切到 mac，R011 落定、R010 归档 |
| 2026-08-31 | `docs\diary\2026-08-31-落地R012对齐清单.md` | 落地 R012 对齐清单 |
| 2026-09-01 | `docs\diary\2026-09-01-新增reader工具与构建修复.md` | 新增 reader 工具、并行会话 Windows 构建修复、转换器本地节保留 |
| 2026-09-01 | `docs\diary\2026-09-01-承接完整迁移与安装形态整改.md` | 承接 ohmypwsh 部署验收自愈完整迁移；用户目录自部署与独立数据目录 |
| 2026-09-01 | `docs\diary\2026-09-01-mac接管M1字段族与真机验证.md` | M1 mac 字段族、R011 六项真机验证全绿、ohmypwsh 已部署工具幂等接管 |
| 2026-09-01 | `docs\diary\2026-09-01-M0数据回流与主权.md` | psd1 Pos 侧一次性回流、pin 平台分列、转换器只校验不再生、kimi 剔除与 31 工具 |
| 2026-09-01 | `docs\diary\2026-09-01-mac完美收敛.md` | 目录型运行时布局实证、extra_bins 多二进制、pin 补齐与升级接管、18 工具三态全等 |
| 2026-09-01 | `docs\diary\2026-09-01-交接收尾与推送.md` | mac 三批次收官，R011 接力刷新与 Windows 回接要点，10 笔提交推送 |
| 2026-09-01 | `docs\diary\2026-09-01-windows回接-平台不适用容忍.md` | Windows 回接全门禁绿；修仅 linux 字段工具致 status 全挂，platform_managed 空态与跳过语义 |
| 2026-09-02 | `docs\diary\2026-09-02-Agent友好IO重构.md` | S003 三格式渲染与结构化错误、self update 三通道与 CI 双通道、WSL msi 与 Docker 接管、rust 接管、M4 heal 移植、taxonomy 定稿、写作规范转换、跨仓 ISSUE 矩阵、停栈实测 |
| 2026-09-02 | `docs\diary\2026-09-02-定位定调与init-doctor.md` | 定位定调（本地本系统的工具与运行时部署管理）；self-deploy 改名 init、verify 流式、doctor 部署异常诊断 |
| 2026-09-05 | `docs\diary\2026-09-05-文档体系evo对齐重建.md` | project-evo 骨架对齐重建：PRD 追溯、proven、ROADMAP、AGENTS 义务表与瘦身、INDEX 磁盘对账、diary 禁字清剿 |
| 2026-09-07 | `docs\diary\2026-09-07-dotfiles吸收与双工具入册.md` | dotfiles 参考吸收：zoxide/sheldon 入册、S004 落档、下载链模式进 D08 设计 |
| 2026-09-08 | `docs\diary\2026-09-08-镜像对账与D08第二批边车锚回落.md` | 镜像对账、D08 第二批边车锚回落、D26 gitleaks 入册、D27 种子自维护 |
| 2026-09-09 | `docs\diary\2026-09-09-Issue清账与双遗留单关单.md` | Issue 清账与双遗留单关单、D28 入册清单化、推送验收与 CI 消音 |
| 2026-09-10 | `docs\diary\2026-09-10-三仓共识落档与claude资产链修复.md` | D29 三仓对齐共识落档回执；claude linux pattern 修复与 catalog_lint pattern 机检（#10 缺口 1）；codex linux 嵌套 bin 根因定位 |
| 2026-09-10 | `docs\diary\2026-09-10-typst入册.md` | D32 typst 入册：三平台 pin v0.15.1（digest 锚加本机实测核验）、真机安装验收、计数 46 改 47 |
| 2026-09-10 | `docs\diary\2026-09-10-软件清单云端化与实时刷新.md` | D33 软件清单云端化：主功能 catalog 下的 status 与 sync 两子功能、TTL 自动刷新、真机旧二进制读到云端新增软件的端到端实证 |
| 2026-09-11 | `docs\diary\2026-09-11-manifest引擎对线三轮补审与修正.md` | manifest 引擎对线三轮补审：L2 管道抽干假超时（M017 实证）、`.cmd` 兜底相对定位、manifest 拉取吞错与同锚不刷、双轨判定粒度与 shim 落点、catalog manifest 字段解析；测试 8 加 4 全绿 |
| 2026-09-12 | `docs\diary\2026-09-12-D41立项更名Ark迁移计划.md` | D41 立项更名 Ark：七点口径与四阶段迁移计划落 PLAN/PRD/GOAL/TODO；oma heal hooks 新形态知识转递（hook 在但 shim 缺失） |

## 错误速查分类

> 位置 `docs\mistakes\`；分类文件 M1xx，行级 M0xx。已有 M102/M105/M106/M101/M103。

| 编号 | 文件 | 覆盖主题 | 行级条目 |
| --- | --- | --- | --- |
| M101 | `docs\mistakes\M101-版本解析与下载-错误.md` | 版本解析与下载错误（REST、cdn、网络、哈希；M007 pin sha 不比 tag；M008 半截缓存复用；M013 滞后二进制复现已修行为；M019 云端 URL 拼成双 scheme 且被静默跳过掩盖；M026 回滚防护只挂显式路径漏默认自动路径） | M007、M008、M013、M019、M026 |
| M102 | `docs\mistakes\M102-解压与安装-错误.md` | 解压与安装错误（九分派、防穿越、幂等；M002 测试沙盒漏 catalog；M004 上游布局变更致展平误判；M017 管道未抽干致 post_install 假超时；M021 杀进程树顺序错致 taskkill 无效；M022 逐通道复制原语应用致 env_set 漏接与语义分叉；M023 直链幂等判据只看悬空致旧 target 陈旧遮蔽；M025 fnm 版本目录用字典序取最大） | M002、M004、M017、M021、M022、M023、M025 |
| M103 | `docs\mistakes\M103-PATH与注册表-错误.md` | PATH 与注册表错误（HKCU、展开、去重；M009 Unix PATH 单槽；M010 死链前缀误伤；M011 写 PATH 未广播） | M009、M010、M011 |
| M104 | 待建 | 文档与命名错误（命名、六态、diary、标题规范） | |
| M105 | `docs\mistakes\M105-工具链与脚本-错误.md` | 工具链与脚本错误（sed、grep、PowerShell、中文路径；M005 Set-Content -NoNewline；M012 种子差集未 HEAD；M015 边车写源目录；M016 cfg 门控面漏跑矩阵；M018 盲切删段连带删掉相邻用例；M020 编辑工具把 CRLF 行写成 LF 致整文件假 diff；M024 一次性脚本删后回仓二犯） | M005、M006、M012、M015、M016、M018、M020、M024 |
| M106 | `docs\mistakes\M106-catalog转换与数据保真-错误.md` | catalog 转换与数据保真错误（转换合并规则、psd1 与 New-ToolDef 分歧、平台字段缺失容忍；M014 同名族资产 digest 错配） | M001、M003、M014 |

迭代规则：踩坑按当前最大号接编 MNNN 进对应分类文件（M0xx 行级、新分类用 M1xx 接编）；一行一事；同根因或同型坑**可合并聚合**进已有条目（保留最早编号与首踩日期，聚合后的正解写全）；反复踩落 `docs\research\`；改「正确处理」不删历史行；新分类文件登记本节。

## 代码文件位置

| 文件 | 职责 |
| --- | --- |
| `src\manifest.rs` | manifest.toml 引擎（R016 B 层 D39）：解析与 schema 拒载、L1 env_set/shims 三平台原语、L2 受控命令超时与失败报告 |
| `src\lib.rs` | crate 根：模块声明与库入口（集成测试链接面） |
| `src\main.rs` | clap CLI 入口与子命令分派（query/pin/install/update/status/init（self-deploy 别名）/self update/verify/heal/doctor/skill；`--llms`；输出纪律与示例元数据在文件顶部） |
| `src\omerr.rs` | 机器可读错误四元组（code/message/hint/exit_code），main 按 exit_code 退出 |
| `src\render.rs` | 单一渲染层：stdout 只走 key=value 数据，组标题走 # 注释行 |
| `src\catalog.rs` | `catalog\tools.toml` 读写、EnvRoot 解析、pin 回写 |
| `src\resolve.rs` | 版本解析三分支（GitHub REST / cdn 模板 / HashiCorp index） |
| `src\download.rs` | 资产下载与缓存复用 |
| `src\checksum.rs` | sha256 校验与官方校验源 |
| `src\install.rs` | 安装主编排（幂等、防穿越、验版本、回写） |
| `src\extract.rs` | 解压/安装九分派 |
| `src\envpath.rs` | 注册表用户 PATH 管理（re-export platform 的跨平台 PATH 管理） |
| `src\platform.rs` | 平台抽象层：EnvRoot 默认路径、PATH 管理、环境变量展开、official 判定、self-deploy 目标 |
| `src\toolver.rs` | 已装版本探测（探测参数与正则读 catalog `probe_args`/`probe_pattern` 字段，D28 迁移；exe 路径解析与 PATH 现查） |
| `src\status.rs` | status 三态对照 |
| `src\selfdeploy.rs` | 自部署到用户程序目录（Windows `%LOCALAPPDATA%\Programs\ome`）+ catalog 同步到用户数据目录 |
| `src\selfupdate.rs` | ome 自升级三通道（dev 滚动 / stable 正式版 / git 源码）：digest 对比后替换自部署目标；官方失败回落镜像对应通道（stable 走 ome/stable，dev 走 ome/dev，段名与通道同名；ome/latest 段已退役）；catalog 同步保留本机 pin |
| `src\vsbuild.rs` | VS Build Tools 接管（evergreen 引导器、gsudo 提权、机器级 PATH、cl.exe 幂等探测；语义见 R001 六） |
| `src\rustup.rs` | Rust 接管（rustup 引导器型：rsproxy 直链 stable 滚动、RUSTUP_HOME/CARGO_HOME 重定位 EnvRoot、cargo sparse 镜像；自 set-rust.ps1 迁移） |
| `src\docker.rs` | Docker Engine 接管（自 set-docker.ps1 迁移：static zip + Windows 服务注册 + daemon.json 合并 + compose 插件 + 机器级 PATH；gsudo 提权；与 vsbuild 差异在有 pin 非 evergreen） |
| `src\verify.rs` | 部署域验收维度注册表（catalog 三态加文件存在判定，`dim=PASS/FAIL/NA` 收割行；流式输出） |
| `src\heal.rs` | 部署维度幂等自愈（install 类原生安装、密钥载体/镜像源 heal-keys/heal-mirror、agent 域休眠、外域只提示、mac-* 别名归一） |
| `src\doctor.rs` | 核心诊断命令两层（D07 三层，D30 收窄去 agent 层：装态对账归 omc、token 归 oma diagnose）加 check 节：环境错误、配置健康（D11）、部署深诊（D12）、网络通连（D13 并行 HEAD、5s 总超时）；verdict=ready/degraded/broken；FAIL 即 exit 1 |
| `tests\cli.rs` | CLI 集成冒烟（离线夹具 catalog，断退出码与 key=value 标记行） |
| `tests\catalog_lint.rs` | catalog 结构机检（D28 入册清单化：在管必有 probe_pattern、正则可编译含捕获组、sha 64 hex；D29 增 pin 资产名必被 asset_pattern 命中；真仓与夹具同规则） |
| `tests\install.rs` | install 链路集成（临时 EnvRoot 沙盒 + 动态 catalog，全程离线：幂等、防穿越） |
| `tests\linux_install.rs` | Linux/macOS 部署集成（真实 GitHub 资产 jq，HOME 沙盒；`cfg(not(windows))` 门控） |
| `tests\golden.rs` | 黄金文件回归（expected oracle 全量比对 stdout，S002 三件套） |
| `tests\real.rs` | 真机闸门（OME_TEST_REAL=1 才跑，对照本机 catalog 与 EnvRoot 部署态） |
| `tests\mirror_fallback.rs` | 镜像兜底链真网闸门（OME_TEST_MIRROR=1 才跑：官方不可达回落 env.ohmygh.com，pin 锚 sha 与 pin 一致、evergreen latest 段与 ome dev 段 sha 与边车一致，D08 两批齐） |
| `tests\common\mod.rs` | 集成测试共享设施（ome 运行 helper 与 expected oracle 断言） |
| `tests\expected\` | 黄金文件 oracle（pin/status 全量 stdout 期望，平台双 oracle：linux/macos 实机冻结；`##` 头注释记来源，约定见 R004 二、4） |
| `tests\fixtures\tools.toml` | 离线测试夹具 catalog（cli 与 install 沙盒共用） |
