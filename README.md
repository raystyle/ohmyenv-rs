# ome

**Oh My Env**：本机跨平台（Windows / Linux / macOS）环境部署管理 CLI，管 40 个工具与运行时
（含 Claude Code、Codex、Grok、Kimi 四家 agent 二进制）的版本解析、下载校验、PATH 注册、
pin 锁定、日常更新与 doctor 三层诊断（系统 / agent / 依赖）。

核心工作流是**检测驱动安装**：doctor 先体检（幂等检测，本地与目标一致即免装），缺口再走
install/deploy 补齐；下载官方渠道失败自动回落 env.ohmygh.com 自建镜像（有 sha 锚才回落，
catalog pin 即锚）。ome 自身装在用户目录（与被管理的 EnvRoot 解耦），自注册 PATH。

四仓生态：ome 管工具、运行时与 agent 二进制下装部署（存量原地纳管：PATH 在位即跳过）；
oma 管 agent 配置、hook 与编排；ohmypwsh 五端总台与密钥；ohmycloud 云端分发镜像基座。

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
| `ome doctor [--json]` | 核心诊断三层（D07）：系统层（os/arch/avx 指令集）、agent 层（四家二进制/版本/token 可用性，不取设置不读环境变量）、依赖层（十类分组统计）；再加环境错误十项与配置健康五项（bunfig/goproxy/cargo 镜像、rust 重定位、遥测开关；判据与 heal 写入动作同源，密钥域归 ohmypwsh 不查） |
| `ome query [tool\|all] [--latest\|--tag\|--version]` | 只解析版本与资产，不下载 |
| `ome install [tool\|all]` | 装入环境目录，不改 PATH；官方渠道失败回落 env.ohmygh.com 镜像（有 sha 锚才回落） |
| `ome deploy [tool\|all]` | 安装 + 注册用户 PATH（默认锁定版本） |
| `ome update [tool\|all]` | 更新到最新版并锁定；agent 类 PATH 在位即跳过（升级走 agent 自更新或 install --force） |
| `ome pin [tool\|all] [--latest\|--version]` | 查看/设置 pin（lock 为别名） |
| `ome status` | 锁定 vs 已安装 vs PATH 三态对照（流式输出） |
| `ome daily [--dry-run] [--include-breaking]` | 日常更新：同主版本自动，跨主版本保留（退出码 2）；agent 类跳过 |
| `ome init` | 安装自身到用户目录、同步 catalog、注册 PATH（`self-deploy` 兼容别名；幂等） |
| `ome package <tool\|all> [--out <dir>]` | 打包为可分发目录（供 scp 与镜像离线装料） |
| `ome verify [--check <维度,...>]` | 部署域验收维度检查（FAIL 即 exit 1） |
| `ome heal [<维度>\|all] [--dry-run]` | 部署维度幂等自愈 |
| `ome self update [--stable\|--git]` | 升级自身三通道（dev 滚动 / stable 正式版 / git 源码） |

## 管理工具名录

> 40 个工具。

> 唯一 pin 源与静态字段权威：`catalog\tools.toml`（十类 taxonomy，节序即类序；清单随 catalog 变动同步）。

| 类 | 工具 |
| --- | --- |
| 智能体依赖（4） | claude、codex、grok、kimi |
| 操作编排依赖（2） | ome（自管条目）、herdr |
| 运行时依赖（7） | pwsh、wsl、docker、dotnet、bun、python、nushell |
| 运行时管理器依赖（2） | fnm（纯 node 运行时管理）、uv（python 运行时管理兼运行时） |
| 编译器依赖（4） | vsbuild（含 C 编译器）、rust、go、zig |
| 多路复用依赖（1） | rmux |
| 远程服务依赖（1） | openssh |
| 密钥安全管理（2） | age、sops |
| 命令工具依赖（16） | git、gh、aria2、7z、gsudo、oscdimg、rg、jq、mq、yq、starship、just、ast-grep、rumdl、shellcheck、zoxide、sheldon |

注：browser-harness 与 reader 暂不接管（2026-09-07 用户裁）；oma（操作编排）与 omcf（运行时衍生）为兄弟仓预留条目待集成；sheldon 上游无 Windows 资产
（Linux/mac 入册，Windows 空态）；shellcheck 仅 Linux 入册。
