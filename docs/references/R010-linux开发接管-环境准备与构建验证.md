# R010：linux 开发接管——环境准备与构建验证

> ome 开发主机从 Windows 切到 Linux（lan-linux 或 WSL）的接管细则：工具链准备、构建验证、平台门控现状、两端验证分工。事实性断言标六态。

## 一、前提与仓库

1. 仓库：`https://github.com/raystyle/ohmyenv`，主分支 `main`（[实证] 2026-08-31 已推送，含 Cargo.lock）。
2. 目标平台仍是 Windows 本机部署管理；Linux 是**开发主机**，不是本期管理对象（[实证] AGENTS.md 定位节）。Linux/mac 被管侧策略为系统标准目录、不进 ohmyenv 目录，属后续扩展（[实证] AGENTS.md 边界节）。

## 二、工具链准备（Linux 侧）

| 工具 | 用途 | 准备方式 |
| --- | --- | --- |
| rustup / cargo | 构建与测试 | rustup 官方脚本装 stable；开发基线 1.97.1（[实证] Windows 侧 cargo 1.97.1 全绿） |
| uv + python | 跑 .tools 三个 md 扫描脚本 | uv 官方脚本；`uv run --script .tools\<名>.py`（[实证] 脚本带 PEP 723 头，无需预装依赖） |
| rumdl | markdown lint | 可选；linux x64 二进制在 ohmypwsh catalog Pos 侧有 pin（[记忆] 待复核资产名） |
| gh | GitHub REST 限流回退 | 可选；匿名解析不依赖（[实证] resolve.rs 仅 403 时回退 gh api） |

## 三、首次构建验证

```bash
git clone https://github.com/raystyle/ohmyenv
cd ohmyenv
cargo build
cargo test
```

验收标准：`cargo build` 零错误；`cargo test` 全绿，Windows 专属测试允许按平台 cfg 跳过（[假设] 标准待 Linux 首跑实证回填）。

## 四、平台门控现状

> 以下为 2026-08-31 在 Windows 侧的源码核查结果。

1. 已有门控（[实证] 2026-08-31 源码核查）：
   - `src\envpath.rs`：注册表读写函数均在 `#[cfg(windows)]` 内，`cfg(not(windows))` 有占位实现。
   - `src\install.rs`：Windows 专属分支有 `#[cfg(windows)]` / `cfg(not(windows))` 对。
   - `tests\real.rs`：真机对齐测试由 `OME_TEST_REAL` 环境变量闸门，未设自动 skip，不依赖 pwsh 存在。
2. 已知待办（[推断] 按代码结构推出，待 Linux 首跑证实）：
   - `Cargo.toml` 的 `winreg = "0.52"` 是无条件依赖，应收进 `[target.'cfg(windows)'.dependencies]`。
   - 若首跑暴露其它 Windows 专属调用（如 msiexec、%VAR% 展开、硬链接语义差异），逐个收编 cfg 门控，不绕过去。
3. 禁改项（[实证] AGENTS.md 边界节）：不为 Linux 构建改动 Windows 行为语义；门控只隔离平台差异，两平台共享逻辑保持单份。

## 五、两端验证分工

| 侧 | 负责 | 命令 |
| --- | --- | --- |
| Linux（开发主机） | 非 Windows 逻辑：解析、下载、校验、catalog、渲染、错误结构的 build + test | `cargo build`、`cargo test` |
| Windows（验收机） | Windows 专属行为：注册表 PATH、msi/7zsfx、self-deploy、真机对齐 ohmyenv.ps1 | `OME_TEST_REAL=1 cargo test`（[实证] 2026-08-31 全绿：status 29/29、query 同 tag、daily 同判定） |

## 六、文档与提交纪律

1. 文档验证三件套在 Linux 同样跑：rumdl check、md-ref-scan、md-heading-scan（[实证] 三脚本与 rumdl 均有 linux 形态；rumdl 若缺可暂缓，两个 py 扫描必跑）。
2. 提交规范不变：`feat:`/`docs:`/`fix:`/`chore:` 前缀 + 中文描述、一事一提交；推送前确认本地全绿（[实证] AGENTS.md 规则 5）。
3. 踩坑与发现按 G003 五步闭环落 docs，跨仓事项（如 grok 的 New-ToolDef 滞后）查 `docs\mistakes\M106` M001（[实证] 已记录前置条件）。
