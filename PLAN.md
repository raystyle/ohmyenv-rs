# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D32 typst 入册（用户 2026-09-10 指令「增加 typst v0.15.1 的安装」，点名 release tag）。

### 依据

- 用户指令点名 `github.com/typst/typst/releases/tag/v0.15.1`；typst 是文档排版系统 CLI（compile / watch / init 子命令出 PDF 与图片），归 cli 类。
- 入册纪律在档（R001 入册 checklist，D28 机检化）：静态字段三平台族、probe_pattern 必填、pin 四键同 tag、sha 与官方源逐字核验、装后探测验证、`seed.py --plan` 域面对账、计数同步。
- 上游 release 无统一校验清单（资产列表无 SHA256SUMS / checksums），锚形态取 GitHub digest（lightpanda D24 先例），并加本机下载实测哈希二次核验。

### 方案骨架

1. **锚核验先行**：release v0.15.1 资产清单逐字抓取；win / linux / mac 三平台资产本机下载算 sha256，与 GitHub digest 逐字对齐（M014 同型防复发：多架构同名族资产行不可错配）。
2. **catalog 节**：`[tools.typst]` 追加 cli 类节尾（节序即类序）：win `zip` 展平（zip 内 `typst-x86_64-pc-windows-msvc/` 单包裹层）、linux `tarxz-bin`（`typst-x86_64-unknown-linux-musl`）、mac `tarxz-bin`（`typst-aarch64-apple-darwin`，ome mac 为 ARM）；probe 走 `typst --version`（实测输出 `typst 0.15.1 (9dfd3a08)`）。
3. **计数与文档同步**：46 改 47（AGENTS 两处、README 三处含类表、SKILL、INDEX、catalog 头注释）；PRD D32、GOAL 锚点与时间线、TODO 行、CHANGELOG Unreleased、diary 一篇。
4. **命令面不动**：入册只加数据，不改 CLI（D10 三原语口径不变）。

### 完成定义

- catalog 有 `[tools.typst]` 节：三平台静态字段齐、pin 四键同 tag、probe_pattern 在档，`catalog_lint` 机检绿。
- Windows 真机 `ome install typst` 幂等二连绿、`ome status typst` 三态齐、探测版本为 0.15.1。
- 计数四处同步为 47；门禁四件套加 cargo test 全绿。

### 验收

- `cargo test` 全绿（含 catalog_lint 真仓与夹具）；`cargo clippy` 干净；`rumdl check .` 与 `.tools` 三扫描绿。
- `uv run --script .tools/seed.py --plan` 域面 diff：typst 三件呈缺种（待 catalog 推送触发 seed-mirror 路线 B 自动入镜）。
- linux / mac pin 按官方 digest 直填，对应机器装后仍走同一 pin 核验；黄金文件不受影响（夹具 catalog 不驻 typst）。
