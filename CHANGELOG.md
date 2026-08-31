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
