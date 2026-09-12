# ark

**Ark（Agent Runtime Kit）**：本机跨平台（Windows / Linux / macOS）环境部署管理 CLI。管 47 个工具与运行时
（含 Claude Code、Codex、Grok、Kimi 四家 agent 二进制）的版本解析、下载校验、PATH 注册、
pin 锁定、更新与 doctor 诊断（系统 / 依赖两层加 check 节）。

> D41 起由 `ome`（Oh My Env）更名而来：命令 `ark`、首发版 1.0.0；旧环境变量
> （`OHMYENV_ROOT`、`OME_CATALOG` 等）与旧部署位读回兼容，`ome` 以部署位别名过渡可用。

- **检测驱动安装**：doctor 先体检，缺口才 install；install 幂等（已装且版本一致即跳过）
- **镜像兜底**：下载官方渠道失败自动回落 env.ohmygh.com 自建镜像，仅当有 sha 锚（catalog pin
  或官方清单）才回落，校验不放松
- **agent 存量纳管**：agent 二进制 PATH 在位即跳过，不重装不迁移
- **输出纪律**：数据走 stdout、提示走 stderr、错误为单行 JSON；`--format kv|json|jsonl` 可选

## 安装

需要 Rust 工具链（[rustup](https://rustup.rs)）。三平台同一套流程：克隆、构建、自部署。

Windows（PowerShell 7）：

```powershell
git clone https://github.com/raystyle/ark-rs
cd ark-rs
cargo build --release
.\target\release\ark.exe init   # 装到 %LOCALAPPDATA%\Programs\ark，自注册用户 PATH（同目录留 ome 别名）
```

Linux / WSL / macOS：

```bash
git clone https://github.com/raystyle/ark-rs
cd ark-rs
cargo build --release
./target/release/ark init      # 装到 ~/.local/bin，PATH 写入当前 shell 配置（zshrc 或 bashrc）
```

重开终端后 `ark doctor` 验证（首次会列出缺口，按提示 `ark install` 补齐）。

几点说明：

- 被管理工具装在环境根目录（EnvRoot）：Windows 默认 `D:\ohmyenv`（无 D: 盘则 `C:\ohmyenv`），
  Linux / macOS 默认 `~/.local/share/ohmyenv`（EnvRoot 物理目录不随更名动）；可用 `--env-root`
  或环境变量 `ARK_ROOT`（读回 `OHMYENV_ROOT`）改
- ark 自身装用户目录，与 EnvRoot 解耦，互不干扰
- 升级自身：`ark self update`（dev 滚动通道；`--stable` 走正式版）

## 使用示例

装工具、看状态、更新、锁定：

```powershell
ark doctor                # 体检：系统 / 依赖两层加 check 节，缺口与修复建议一目了然
ark install rg            # 装单个工具：下载、sha 校验、解压、PATH 注册一次完成
ark install               # 全量安装（幂等：已装且版本一致即跳过）
ark status                # 三态对照：锁定版本 / 已装版本 / PATH 是否在位
ark query ffmpeg --latest # 只查最新版与资产，不下载
ark update                # 全部拉云端最新安装（锁定归云端数据面，不回写）
ark update herdr          # 更新单个工具
ark pin                   # 查看全部版本锁定
```

清单同步（云端软件清单，新增软件不用换二进制）：

```powershell
ark catalog               # 看解析面与 manifest 面（在位/锚/年龄/签名）、云端锚与同步态（status 简写）
ark catalog sync          # 立即从 env.ohmygh.com 刷新本机运行态清单（边车 sha 加 minisign 签名双校验）
```

验收与自愈：

```powershell
ark verify                # 部署域验收：逐维度 PASS/FAIL，FAIL 则退出码非零（可进脚本）
ark heal --dry-run        # 预览可自愈的部署维度
ark heal                  # 执行自愈（PATH 修复、镜像源补写等，幂等）
```

## 命令速查

| 命令 | 说明 |
| --- | --- |
| `ark doctor` | 两层诊断（系统 / 依赖）加 check 节（环境错误、配置健康、部署深诊、网络通连）；verdict=ready/degraded/broken |
| `ark query [名]` | 解析版本与资产，不安装；`--latest` / `--tag` / `--version` 定向 |
| `ark install [名]` | 下载解压到 EnvRoot，注册 PATH、写注册表与配置；省略则全量 |
| `ark update [名]` | 拉云端最新并安装（不回写锁定，锁定归数据面）；省略则全量；agent 类 PATH 在位即跳过 |
| `ark pin [名]` | 查看 / 设置版本锁定（lock 为别名） |
| `ark status` | 锁定 / 已安装 / PATH 三态对照（流式输出） |
| `ark init` | 自部署：安装自身、同步 catalog、注册 PATH、接管旧 ome 部署位（幂等） |
| `ark verify` | 部署域验收维度检查，FAIL 即 exit 1 |
| `ark heal [维度]` | 部署维度幂等自愈，`--dry-run` 预览 |
| `ark skill` | 生成本机实装清单 SKILL（落数据目录） |
| `ark catalog [status\|sync]` | 运行态软件清单两件：catalog 面（解析面、云端锚、同步态）与 manifest 面（在位、本地锚、年龄、云端锚对比、签名）；sync 立即从云端刷新两件（默认 TTL 24h 自动刷新，`ARK_CATALOG_TTL` / `ARK_OFFLINE=1` 可关，旧名 `OME_*` 读回） |
| `ark self update` | 升级自身（dev / stable / git 三通道） |

全量输出契约（字段与退出码）见 `docs\references\R013-Agent友好IO契约-输出格式退出码与冻结面.md`。

## 管理工具名录

47 个工具；清单数据权威在云端（ohmycloud catalog-seed 与镜像三件套），本仓持格式契约（R001）与消费逻辑。

| 类 | 工具 |
| --- | --- |
| 智能体依赖（4） | claude、codex、grok、kimi |
| 操作编排依赖（2） | ome（自管条目，D41 起引擎双接受 `ome-self`/`ark-self`，条目更名归数据面）、herdr |
| 运行时依赖（7） | pwsh、wsl、docker、dotnet、bun、python、nushell |
| 运行时管理器依赖（2） | fnm（node）、uv（python） |
| 编译器依赖（4） | vsbuild（含 C 编译器）、rust、go、zig |
| 多路复用依赖（1） | rmux |
| 远程服务依赖（1） | openssh |
| 密钥安全管理（3） | age、sops、gitleaks |
| 命令工具依赖（22） | git、gh、aria2、7z、gsudo、oscdimg、rg、jq、mq、yq、starship、just、ast-grep、rumdl、shellcheck、zoxide、sheldon、ffmpeg、rclone、reader、lightpanda、typst |
| 运行时衍生（1） | browser-harness（bin 名 bh） |

注：平台空态如实表达。sheldon 上游无 Windows 资产（win 空态）；shellcheck 仅 Linux 入册；
ffmpeg 官方 mac 构建仅 Intel（ark mac 为 ARM 故空态）；lightpanda 上游无 Windows 构建。
