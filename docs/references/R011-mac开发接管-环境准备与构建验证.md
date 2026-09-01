# R011：mac 开发接管——环境准备与构建验证

> 开发主机由 WSL/Linux 切换到本机 mac。ome 的 `#[cfg(not(windows))]` 分支同时覆盖 Linux 与 macOS，因此代码层面以 Linux 实现为基线，mac 侧重点在于工具链、构建验证与目录/PATH 习惯差异。本文件涵盖：工具链准备、首次构建验证、mac 目录/PATH 策略、与 Linux/Windows 的验证分工。事实性断言标六态。

## 一、前提与仓库

1. 仓库：`https://github.com/raystyle/ohmyenv-rs`（2026-09-01 由 ohmyenv 改名），主分支 `main`，含 Cargo.lock（[实证]）。
2. 开发主机：本机 mac（Apple Silicon 或 Intel）。clone 落 mac 原生文件系统（家目录，如 `~/ohmyenv-rs`），不放外置/网络卷：权限、大小写敏感、Spotlight 索引都可能引发异常（[经验]）。
3. 目标平台仍包括 Windows 与 Linux；mac 自身同时承担「开发主机」与「被管理目标」两个角色（[推断] 由用户决策）。
4. **接力状态（2026-09-01）**：立项批次已收盘（新址 clone、安装形态整改、旧 clone 清理、R012 降级、vsbuild 接管），下一步 **M0 数据主权与回流**（见 ohmypwsh 仓库 P0026 方案）。M0 的数据源是 ohmypwsh 的 `scripts\catalog.psd1` Pos 侧——mac 上需同时 clone ohmypwsh 仓库（或至少取到该文件），且回流完成前该文件 Pos 侧冻结禁改（唯一现存 linux/mac pin 来源）。

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
git clone https://github.com/raystyle/ohmyenv-rs ~/ohmyenv-rs
cd ~/ohmyenv-rs
cargo build
cargo test
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
uv run --script .tools/md-ref-scan.py
uv run --script .tools/md-heading-scan.py
```

验收标准：`cargo build` 零错误；`cargo test` 全绿（2026-09-01 基线 74 项），Windows 专属测试按平台 cfg 跳过（[推断] 与 WSL 行为一致，待 mac 真机验证）；vsbuild 相关测试在 mac 上按 cfg 门控跳过或走纯函数路径（[实证] vsbuild 安装分支非 Windows 即报错，测试仅覆盖参数与探测纯函数）。Windows 交叉回归建议 `cargo check --target x86_64-pc-windows-gnu`（mac 需 `rustup target add`；Windows 主机侧无该 target 且 R010 已实证其交叉 check 不可行，故 gnu 交叉回归以 mac/Linux host 为准）。

## 四、mac 目录/PATH/提取策略

1. **ome 自身（2026-09-01 安装形态整改后）**：自部署二进制 `~/.local/bin/ome`（`self-deploy` 复制并注册 PATH）；元数据与部署态 catalog 在 `~/Library/Application Support/ohmyenv`（`dirs::data_local_dir()/ohmyenv`，三平台统一），`self-deploy` 会把仓库 `catalog\tools.toml` 同步到 `<数据目录>/catalog/`，部署态二进制按解析四级（OME_CATALOG、exe 相邻、cwd、数据目录）找 catalog，不依赖 clone 目录（[实证] Windows 侧同构实现已验证，mac 侧待真机）。
2. **被管理软件安装目录**：默认单二进制绿色工具安装到 `~/.local/bin`，与 Linux 一致；复杂运行时按 `linux_dir`（mac 复用 `linux_*` 字段族，M1 将增设 mac 专属字段族）。
3. **PATH 管理**：默认 shell 在 mac 上通常为 zsh，`platform.rs` 按 `$SHELL` 检测并写入 `~/.zshrc`；若使用 bash 则回退到 `~/.bashrc`（[推断] 由 `platform.rs::profile_path()` 逻辑）。
4. **extract 类型**：与 Linux 一致，跨平台 `zip`/`targz`/`copy`/`single`，Linux/mac 新增 `targz-bin`/`tarxz-bin`；Windows 专属类型（含 2026-09-01 新增的 `vsbuild`）在 mac 下返回明确错误。

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
4. `ome self-deploy` 后从任意目录跑 `ome status`，catalog 命中 `~/Library/Application Support/ohmyenv/catalog/tools.toml`（部署态独立解析）。
5. `ome package fnm --out ./deploy` 输出 `./deploy/fnm/bin/fnm`。
6. `cargo check --target x86_64-pc-windows-gnu` 通过（需先 `rustup target add`）。

## 七、文档与提交纪律

1. 文档验证三件套在 mac 同样跑：rumdl check、md-ref-scan、md-heading-scan（[推断] 与 WSL 同理）。
2. 提交规范不变：`feat:`/`docs:`/`fix:`/`chore:` 前缀 + 中文描述、一事一提交；推送前确认本地全绿（[实证] AGENTS.md 规则 5）。
3. 踩坑与发现按 G003 五步闭环落 docs；mac 特有问题接编 `docs/mistakes` 并登记 `INDEX.md`。
