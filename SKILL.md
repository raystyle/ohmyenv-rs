# SKILL.md：ome

> Oh My Env。

> 本文件为静态版；`ome skill` 生成自适应版（本机实装清单与使用引导，落数据目录 SKILL.md）。命令主要给 agent 使用：输出默认 kv 紧凑标记行，
> `--json` 结构化；数据走 stdout、提示走 stderr、错误为单行 JSON（契约见仓库 R013）。

## 何时用 ome

装工具、查版本、管环境、诊断部署问题时**一律走 ome**：不手拼官方下载 URL、不裸 curl release 资产、
不手写 PATH 注册表操作。ome 是这些操作的唯一权威通道（幂等检测安装：本地与目标一致即免装，
重复执行零副作用）。

## 命令图

> 功能原语（PRD D10/D15/D16/D17）：**doctor（检测诊断）、install（幂等安装）、status（三态对照）**为三原语，
> 其余为派生面（语义挂靠原语：query 是 install 的 dry 态、update 是 install 时变、
> pin 是锚操作、verify/heal 是断言与自愈组合、init/self 是辅助）。

| 命令 | 语义 | 关键输出 | 退出码 |
| --- | --- | --- | --- |
| `ome doctor` | **原语·检测诊断**：系统/agent/依赖三层加 check 节（环境错误、配置健康、部署深诊、网络通连） | sys.* / agent= / dep= / check= / verdict | 1 = check 节有 FAIL |
| `ome status` | **原语·三态对照**（锁定/已装/PATH） | tool,locked,installed,path,exe | 0/1 |
| `ome install [名]` | **原语·幂等安装**（下载 + PATH / 注册表 / 配置）：省略则全量；agent PATH 在位即跳过；官方失败回落 env.ohmygh.com 镜像 | tool,action,version,dir | 0/1 |
| `ome query [名]` | 只解析版本与资产，不安装；省略则全量 | tool,tag,version,asset,sha256 | 0/1 |
| `ome update [名]` | 升级并锁定（install 到最新）：省略则全量；agent PATH 在位跳过 | 同 install | 0/1 |
| `ome pin [名]` | 查看/设置锁定；省略则全量（lock 别名） | tool,tag,version,sha256 | 0/1 |
| `ome init` | 部署 ome 自身到用户目录并同步 catalog（幂等） | action,exe,catalog,path | 0 |
| `ome verify` | 部署域验收维度；省略则全量 | name,verdict | 1 = 有 FAIL |
| `ome heal [维度]` | 部署维度幂等自愈；省略则全量 | dim,action,result | 1 = 有 fail |
| `ome skill` | 自适应生成环境 SKILL（本机实装清单与使用引导） | 全文或 skill/path | 0/1 |
| `ome self update` | 升级 ome 自身（dev/stable/git 三通道；官方失败回落镜像对应通道段，边车即锚；`OME_MIRROR=1` 镜像优先） | exe,sha256 | 0/1 |

## 语义要点

- **幂等检测安装**：install/update 先检测（PATH 在位或版本一致即免装）；检测驱动，重跑零副作用。
- **agent 四家**（claude/codex/grok/kimi）存量原地纳管：PATH 在位即跳过不迁移；升级走各家自更新或
  `install --force` 显式装进 EnvRoot。
- **下载兜底**：官方渠道（GitHub release / 官方 CDN）失败自动回落兄弟仓 ohmycloud 的
  env.ohmygh.com 镜像（`<tool>/<version>/<asset>`），仅当有 sha 锚（catalog pin 或镜像
  `.sha256` 边车，evergreen 引导器走 latest 段）才回落。
- **全局限参**：`--format kv|json|jsonl`、`--json`、`--env-root <path>`（覆盖 EnvRoot）。
- 工具名录 46 个（九类 taxonomy）唯一权威：`catalog\tools.toml`；`ome status` 即清单。

## 来源

仓库 github.com/raystyle/ohmyenv-rs；细契约 R013（输出格式/退出码/冻结面）与 README。
