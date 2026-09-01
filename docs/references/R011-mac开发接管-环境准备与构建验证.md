# R011：mac 开发接管——环境准备与构建验证

> 开发主机由 WSL/Linux 切换到本机 mac。ome 的 `#[cfg(not(windows))]` 分支同时覆盖 Linux 与 macOS，因此代码层面以 Linux 实现为基线，mac 侧重点在于工具链、构建验证与目录/PATH 习惯差异。本文件涵盖：工具链准备、首次构建验证、mac 目录/PATH 策略、与 Linux/Windows 的验证分工。事实性断言标六态。

## 一、前提与仓库

1. 仓库：`https://github.com/raystyle/ohmyenv`，主分支 `main`（[实证] 2026-08-31 已推送，含 Cargo.lock）。
2. 开发主机：本机 mac（Apple Silicon 或 Intel）。clone 落 mac 原生文件系统（家目录，如 `~/ohmyenv`），不放外置/网络卷：权限、大小写敏感、Spotlight 索引都可能引发异常（[经验]）。
3. 目标平台仍包括 Windows 与 Linux；mac 自身同时承担「开发主机」与「被管理目标」两个角色（[推断] 由用户 2026-08-31 决策）。

## 二、工具链准备

> mac 内的工具准备清单（不依赖 Linux/Windows 侧同名工具）。

| 工具 | 用途 | 准备方式 |
| --- | --- | --- |
| rustup / cargo | 构建与测试 | rustup 官方脚本装 stable（[经验] mac 上 Homebrew 的 rust 版本常滞后，推荐 rustup） |
| uv + python | 跑 `.tools` 三个 md 扫描脚本 | uv 官方脚本；`uv run --script .tools/<名>.py`（[推断] 脚本带 PEP 723 头，与 Linux 同理） |
| rumdl | markdown lint | 可选；mac 形态待从 ohmypwsh catalog 取 AssetMac 后验证（[记忆] 待复核） |
| gh | GitHub REST 限流回退 | 可选；匿名解析不依赖（[实证] resolve.rs 仅 403 时回退 gh api） |

## 三、首次构建验证

```bash
git clone https://github.com/raystyle/ohmyenv ~/ohmyenv
cd ~/ohmyenv
cargo build
cargo test
```

验收标准：`cargo build` 零错误；`cargo test` 全绿，Windows 专属测试按平台 cfg 跳过（[推断] 与 WSL 行为一致，待 mac 真机验证）。建议同时跑 `cargo check --target x86_64-pc-windows-gnu` 保持 Windows 交叉编译回归。

## 四、mac 目录/PATH/提取策略

1. **ome 元数据目录**：`~/Library/Application Support/ohmyenv`（`dirs::data_dir()` 在 mac 上的返回值），含 `cache/`、`logs/`（[推断] 由 `platform.rs` 使用 `dirs::data_dir()`，与 Linux 的 `~/.local/share/ohmyenv` 等价）。
2. **软件安装目录**：默认单二进制绿色工具安装到 `~/.local/bin`；mac 上也按此约定，与 Linux 一致（[推断] 由用户 2026-08-31 要求「Linux 或 MAC 是用户目录」）。
3. **PATH 管理**：默认 shell 在 mac 上通常为 zsh，`platform.rs` 按 `$SHELL` 检测并写入 `~/.zshrc`；若使用 bash 则回退到 `~/.bashrc`（[推断] 由 `platform.rs::profile_path()` 逻辑）。
4. **extract 类型**：与 Linux 一致，跨平台 `zip`/`targz`/`copy`/`single`，Linux/mac 新增 `targz-bin`/`tarxz-bin`；Windows 专属类型在 mac 下返回明确错误。

## 五、与 Linux/Windows 的验证分工

| 侧 | 负责 | 命令 |
| --- | --- | --- |
| mac（开发主机） | 非 Windows 逻辑：解析、下载、校验、安装、PATH、status 的 build + test | `cargo build`、`cargo test` |
| Linux/WSL（验证机） | Linux 本机部署闭环验证（保留，防止 mac 与 Linux 行为分叉） | `cargo test`（[记忆] 2026-08-31 已全绿） |
| Windows（验收机） | Windows 专属行为：注册表 PATH、msi/7zsfx、self-deploy、真机对齐 ohmyenv.ps1 | `OME_TEST_REAL=1 cargo test`（[记忆] 2026-08-31 全绿） |

## 六、mac 真机待验证项

> 以下项需在本机 mac 跑过后回填为 [实证]。

1. `cargo build` / `cargo test` 在 mac 上全绿。
2. `ome install jq --latest` 成功，二进制落在 `~/.local/bin`，`~/.zshrc` 正确写入 PATH 标记块。
3. `ome deploy jq` 后新 shell 能直接调用 `jq`。
4. `ome package fnm --out ./deploy` 输出 `./deploy/fnm/bin/fnm`。
5. `cargo check --target x86_64-pc-windows-gnu` 仍通过。

## 七、文档与提交纪律

1. 文档验证三件套在 mac 同样跑：rumdl check、md-ref-scan、md-heading-scan（[推断] 与 WSL 同理）。
2. 提交规范不变：`feat:`/`docs:`/`fix:`/`chore:` 前缀 + 中文描述、一事一提交；推送前确认本地全绿（[实证] AGENTS.md 规则 5）。
3. 踩坑与发现按 G003 五步闭环落 docs；mac 特有问题接编 `docs/mistakes` 并登记 `INDEX.md`。
