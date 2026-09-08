# ome

**Oh My Env**：本机跨平台（Windows / Linux / macOS）环境部署管理 CLI，管 46 个工具与运行时
（含 Claude Code、Codex、Grok、Kimi 四家 agent 二进制）的版本解析、下载校验、PATH 注册、
pin 锁定、日常更新与 doctor 三层诊断（系统 / agent / 依赖）。

核心工作流是**检测驱动安装**：doctor 先体检（幂等检测，本地与目标一致即免装），缺口再走
install 补齐；下载官方渠道失败自动回落 env.ohmygh.com 自建镜像（有 sha 锚才回落：
catalog pin 即锚，evergreen 引导器以镜像 `.sha256` 边车为锚）。ome 自身装在用户目录
（与被管理的 EnvRoot 解耦），自注册 PATH。

独立仓库：ome 管本机工具、运行时与 agent 二进制下装部署（存量原地纳管：PATH 在位即跳过）。
**ohmycloud** 是资源分发基建兄弟项目（env.ohmygh.com 镜像，官方渠道失败回落）。
oma 管 agent 配置、hook 与编排。成功标准是命令在部署系统上功能完整。

## 快速开始

Windows：

```powershell
cargo build --release
.\target\release\ome.exe init   # 安装到用户目录 %LOCALAPPDATA%\Programs\ome，自注册用户 PATH
ome status
```

Linux / WSL：

```bash
cargo build --release
./target/release/ome init        # 安装到 ~/.local/bin，自注册 PATH（写入 ~/.bashrc）
ome status
```

## 命令

全局 `--format kv|json|jsonl`（默认 kv）与 `--json` 简写；数据走 stdout、提示走 stderr、
错误为单行 JSON；输出字段与退出码契约见 `docs\references\R013-Agent友好IO契约-输出格式退出码与冻结面.md`。

| 命令 | 说明 |
| --- | --- |
| `ome doctor [--json]` | 核心诊断三层（D07）加 check 节：环境错误、配置健康（D11）、部署深诊（D12 rust/vsbuild）、网络通连（D13）；verdict=ready/degraded/broken；FAIL 即 exit 1 |
| `ome query [名] [--latest\|--tag\|--version]` | 只解析版本与资产，不安装；省略则全量 |
| `ome install [名]` | 下载解压到环境目录，并注册 PATH、写注册表与配置；省略则全量；官方渠道失败回落 env.ohmygh.com 镜像（有 sha 锚才回落：pin 或 latest 段边车） |
| `ome update [名]` | 更新到最新版并锁定（即 install 到最新）；省略则全量；agent 类 PATH 在位即跳过（升级走 agent 自更新或 install --force） |
| `ome pin [名] [--latest\|--version]` | 查看/设置 pin；省略则全量（lock 别名） |
| `ome status` | 锁定 vs 已安装 vs PATH 三态对照（流式输出） |
| `ome init` | 安装自身到用户目录、同步 catalog、注册 PATH（`self-deploy` 兼容别名；幂等） |
| `ome verify [--check <维度,...>]` | 部署域验收维度检查；省略则全量（FAIL 即 exit 1） |
| `ome heal [维度] [--dry-run]` | 部署维度幂等自愈；省略则全量 |
| `ome skill` | 自适应生成环境 SKILL（本机实装清单，落数据目录） |
| `ome self update [--stable\|--git]` | 升级自身三通道（dev 滚动 / stable 正式版 / git 源码）；官方 API 失败回落镜像边车锚（dev 走 `ome/dev` 段）；`OME_MIRROR=1` 镜像优先跳官方 |
| `ome --llms` | 打印紧凑命令清单后退出（agent 发现入口，无需 catalog） |

## 管理工具名录

> 46 个工具。

> 唯一 pin 源与静态字段权威：`catalog\tools.toml`（九类 taxonomy，节序即类序；清单随 catalog 变动同步）。

| 类 | 工具 |
| --- | --- |
| 智能体依赖（4） | claude、codex、grok、kimi |
| 操作编排依赖（2） | ome（自管条目）、herdr |
| 运行时依赖（7） | pwsh、wsl、docker、dotnet、bun、python、nushell |
| 运行时管理器依赖（2） | fnm（纯 node 运行时管理）、uv（python 运行时管理兼运行时） |
| 编译器依赖（4） | vsbuild（含 C 编译器）、rust、go、zig |
| 多路复用依赖（1） | rmux |
| 远程服务依赖（1） | openssh |
| 密钥安全管理（3） | age、sops、gitleaks |
| 命令工具依赖（21） | git、gh、aria2、7z、gsudo、oscdimg、rg、jq、mq、yq、starship、just、ast-grep、rumdl、shellcheck、zoxide、sheldon、ffmpeg、rclone、reader、lightpanda |
| 运行时衍生（1） | browser-harness（bin 名 bh，npm-tgz 通道） |

注：browser-harness 与 reader 2026-09-08 重入册（撤 09-07 暂不接管裁；browser-harness 仓已重写 TS，走 npm-tgz 通道，需 node 与 npm 在 PATH）；oma（操作编排）与 omcf（运行时衍生）为兄弟仓预留条目待集成；sheldon 上游无 Windows 资产
（Linux/mac 入册，Windows 空态）；shellcheck 仅 Linux 入册；ffmpeg 官方 mac 构建（evermeet.cx）仅 Intel，ome mac 为 ARM 故空态；lightpanda 上游无 Windows 构建（win 空态，linux/mac 入册）。
