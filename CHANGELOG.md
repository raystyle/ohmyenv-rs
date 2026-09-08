# CHANGELOG

> 版本里程碑。SemVer `vMAJOR.MINOR.PATCH`。

## [Unreleased]

- 升级（B 面验收开关）：`OME_MIRROR=1` self update 镜像优先（跳官方 API 直取边车锚，锚语义不变），供五端断源验收与未来默认切自建过渡。
- 分发（D08 第二批收官，ohmyenv-rs#7 关单）：ohmycloud#9 补种 `ome/dev` 段后补 self update dev 通道断源锚链用例（锚取 ome/dev 边车，产物 sha 逐字一致），`OME_TEST_MIRROR` 7/7 绿。
- 分发（D08 第二批，ohmyenv-rs#7）：evergreen 引导器（rust / vsbuild）官方失败回落镜像 latest 段，`.sha256` 边车即信任锚（先边车后资产，边车取不到拒绝无校验下载）；selfupdate 边车权威统一至 download 层；`OME_TEST_MIRROR` 6/6 绿（zoxide linux 回归、ffmpeg HEAD 在位）。
- 分发（D21）：向 ohmycloud 派三面对齐 ISSUE（#8）：ome 自身与在管软件的分发、更新、安装。
- catalog：回填 zoxide / ffmpeg `linux_sha256`（官方资产哈希与 GitHub digest 一致）。
- 分发（D20）：种子清单用 GitHub ISSUE 向 ohmycloud 派任务并对齐（R014；ohmycloud#5 差集闭环，OME_TEST_MIRROR 绿）。
- 定位（D19）：ohmycloud 是资源分发基建兄弟仓（env.ohmygh.com）；ome 官方失败回落该镜像。
- 定位（D18）：独立仓，不再依赖 ohmypwsh；真机对照为本机 catalog 与 EnvRoot；本机命令面可跑。
- 命令面（D17）：去掉 `package`。
- 命令面：`install` / `update` / `query` / `pin` / `heal` / `verify` 省略名即全量。
- 命令面（D16）：去掉 `daily`；升级一律 `ome update`。
- 命令面（D15）：去掉 `deploy`；`ome install` 一次完成下载、PATH、注册表与配置；`update` 为 install 到最新。
- 入册 ffmpeg（D14）：Windows GyanD/codexffmpeg essentials 9.0.1，Linux BtbN n9.0 GPL 静态，mac ARM 空态（官方 evermeet 仅 Intel）。
- 审查缺陷：pin sha 必须同 tag；install 不污染旧 pin sha；下载 `.part` 提交；CDN 显式版本重算 tag；self update 镜像按通道、替换部署位、catalog 同步保留 pin。
- doctor：probe-fail 看文件存在；死链按路径分量；rust 走 EnvRoot rustup；D13 5s 总超时；网络 WARN 不单独 degraded。
- POSIX PATH 标记块多目录；mac 只认 `mac_exe`；verify localbin16 去掉 vault。
- 立项：自 ohmypwsh `ohmyenv.ps1` 剥离本机 Windows 环境部署管理为 Rust CLI。
- 八命令落地：query / install / deploy / update / pin / status / daily / self-deploy。
- catalog 数据层：`.tools\import-catalog.ps1` 生成 `catalog\tools.toml`（29 工具唯一 pin 源，合并规则对齐 Get-EnvLock）。
- incurs 选型研究（S001）：不迁移，吸收机器可读错误、单一渲染层、帮助元数据化三模式。
- 真机对齐：status 29/29 与 ohmyenv.ps1 逐项一致，query 同 tag（OME_TEST_REAL 闸门测试）。
- 文档体系自 ohmyagents 平移：AGENTS 四段式、G001-G003、R001/R004/R005/R008/R009、.tools md 三件套。
- 开发接管准备：R010 落定 Linux 开发主机的工具链准备、构建验证、平台门控现状与两端验证分工。
- 管理域收窄：智能体（codex/claude/grok）安装剥离出 ome（归 ohmyagents/ohmypwsh），工具名录 29 降至 26，转换器显式剔除。
- Linux 本机部署支持：platform.rs 平台抽象层、catalog linux_* 字段、package 命令、tests/linux_install.rs。
- 开发主机接管：R010（WSL）归档，R011（mac）落定。
- 新增工具 reader（raystyle/reader_rs v0.1.0）：ome 本地名录首个自增工具（27 个），转换器保留本地节；真机 deploy 验证 locked=installed=0.1.0、path=true。
- 承接 ohmypwsh 完整迁移：M0 数据主权（psd1 单向回流、pin 平台分列、转换器改只校验）；M2 四端齐（WSL/lan-linux/lan-win/mac，package 打包下发）；M3 `ome verify` 部署域验收（维度注册表、流式输出）；M4 `ome heal` 自愈移植（42 键四类归宿）。
- `ome self update` 三通道（dev 滚动 / stable 正式版 / git 源码）与 CI 双通道路由，五端自服务闭环。
- Agent 友好 IO（S003）：全局三格式渲染层（kv/json/jsonl）、结构化错误 stderr 单行 JSON、字段序稳定。
- 新接管：rust（rustup.rs 建模，rsproxy 镜像与 EnvRoot 重定位）、Docker Engine（服务注册与 compose 插件）、Windows OpenSSH（MSI 型）、VS Build Tools（evergreen 引导器）、go/zig（cdn 直链）。
- `ome doctor` 部署异常诊断九项；self-deploy 改名 init（兼容别名）。
- 七类 taxonomy 定稿，37 工具（含 ome 自管条目；oma/omcf 预留）。
- 写作规范 G001 v2（四类禁字符硬禁令）与文档门禁四件套（本地与 CI 同口径）。
- 文档体系对齐 project-evo 骨架：补 PRD（D 编号全量追溯）、docs\proven、ROADMAP、AGENTS 文档义务表，INDEX 以磁盘为唯一事实源对账重整（evo check 13 项全绿）。
