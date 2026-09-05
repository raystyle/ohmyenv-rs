# CHANGELOG

> 版本里程碑。SemVer `vMAJOR.MINOR.PATCH`。

## [Unreleased]

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
