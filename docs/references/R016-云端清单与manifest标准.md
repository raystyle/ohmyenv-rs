# R016：云端清单与 manifest 标准

> 角色：标准（**v0.2 正式版**，2026-09-11 定稿，D39；v0.1 草案经 omc 评审三待定点全裁 CONFIRM）。ome 云端软件**清单（catalog）**与**安装 manifest** 的数据标准：字段与 DSL 规范、信任与版本化、跨仓执行分工。**制定在 ome 仓（消费引擎定义契约），维护与数据运营在 omc（云端管理全平台软件清单、分发、部署逻辑，用户裁 2026-09-11）**；流程面见 R015，研究依据见 S007。字段语义底册吸收 R001。

## 一、总则

1. **两件分离**：catalog（`tools.toml`：资产、版本、pin、探测）与 manifest（`manifest.toml`：安装配置部署逻辑）关注点分离、独立演进、同批同签发布（云端 `ome/catalog/` 下三件套扩为 catalog 与 manifest 各带边车加 `.minisig`，签发由 omc catalog-seed 流水）。
2. **全平台第一约束**（S007 五节四原则）：原语优先（三平台语义引擎统一保证）、受控命令三键齐备 lint、平台不适用容忍（M003 空态语义）、平台矩阵即验收（R004 三.6）。
3. **信任链复用 D34**：minisign 端到端签名，签名清单即信；lint（omc 侧 vitest 与 ome 侧 catalog_lint 同规则）装前静态可验。
4. **版本化**：两件各带 `schema_version` 字段（`1` 起）；引擎遇高版本拒载并提示升级 ome（前进兼容红线：不猜语义）。

## 二、A 层：catalog schema

字段语义以 R001 为底册（吸收为标准实现注记），标准面新增：

| 域 | 字段 | 规范 |
| --- | --- | --- |
| 元 | `schema_version` | 正整数；缺省视 `1` |
| 资产解析 | `repo`/`asset_pattern`/`sums_*`/`cdn_*`/`version_pattern` 平台族 | 同 R001；pattern 必命中 pin 资产名（lint 规则） |
| pin | `tag`/`version`/`asset`/`sha256` 平台族 | 四键同 tag；sha 64 hex 大写；锁定单源数据面（D37） |
| 布局 | `extract`/`dir`/`bin`/`exe`/`extra_bins` 平台族 | 同 R001 九分派 |
| 探测 | `probe_args`/`probe_pattern` | 在管必有；正则可编译含第 1 捕获组 |
| manifest 引用 | `manifest` | 可选；指向 `manifest.toml` 节键（默认同名节） |

## 三、B 层：manifest DSL 三层

**L1 声明原语**（无脚本，静态可 lint；三平台语义由 ome 引擎统一实现，每原语须过三平台矩阵才准入标准）：

| 原语 | 形态 | win 语义 | POSIX 语义 |
| --- | --- | --- | --- |
| `env_set` | 键值表 | 注册表 HKCU（platform.rs 通道） | profile 标记块 |
| `shims` | 别名表（源加目标名） | `.cmd` shim 经 `cmd /c` 拉起 | 符号链接 `~/.local/bin` |
| `persist`（**reserved**） | 目录表（相对安装目录） | **v1 不启用**：与 oma（端上 agent 治理）加 omc（金库与 manifest 运营）配置域重叠，且无真实工具需求；字段名与语义保留，真需求以 schema_version 递增引入（omc 评审裁） | 同左 |
| `machine_path`/`elevate` | 布尔 | 触发引擎内建（HKLM 加 gsudo） | 空态容忍（M003） |
| `service` | 服务描述 | 触发引擎内建服务注册 | systemd 用户单元（按需准入） |

**L2 受控命令数组**：分平台键 `post_install.win`/`.linux`/`.mac`，值为 argv 数组（非 shell 字符串，无元字符解释）；三键或显式 `skip` 齐备才过 lint；逐条执行、超时 300s 杀进程、失败**只报不回滚**且报告含退出码与 stdout/stderr 尾行（omc 评审裁）。

**L3 任意脚本不进**：例外场景走引擎内建原语由数据字段触发（S007 三节取舍）。

## 四、跨仓执行分工

| 面 | omc（维护与运营） | ome（制定与执行） |
| --- | --- | --- |
| catalog 与 manifest 数据 | 唯一权威：条目、pin、DSL 内容维护 | 消费；改消费面时以标准 herdr 知会 |
| lint 门 | vitest 四规则加 DSL 语法机检（CI 首步） | catalog_lint 同规则（夹具回归） |
| 签名与发布 | catalog-seed 流水（两件各三件套） | 端上三重门与巡检 |
| 引擎 | 无 | manifest 解释器与原语三平台实现 |

## 五、演进治理

- 标准变更（新原语、新字段）由 ome 仓 PR 修订本文件，`schema_version` 递增；omc 数据面跟进 lint；旧版本数据按兼容窗口保留。
- 专用模块（vsbuild/rustup/docker/msi）渐进迁移为原语组合（按工具渐进，S007 四.3 默认），迁移完成一个撤一个内建分支。

## 六、定案与实现状态

三待定点已裁（2026-09-11 omc 评审回执，全 CONFIRM）：persist 不入 v1 且字段 reserved；L2 超时 300s 加只报不回滚加尾行退出码报告；manifest 单件 `manifest.toml`（粒度瓶颈时以 schema_version 变更窗口拆分）。

引擎首波已落（ome 仓）：`src/manifest.rs`（解析与 schema 版本拒载、L1 env_set/shims 三平台、L2 执行链含超时与失败报告）；install 双轨接线（manifest 节优先、无节内建回退，omc 数据上线后撤内建）；catalog sync 扩拉 manifest 三件套（同锚同签，云端未上线 404 静默跳过）；lint 扩 manifest 面（解析、三键齐备、catalog 引用一致性）；fixtures 样例（pwsh/dotnet env_set、bun shims、demo post_install）。待 omc：catalog-seed 扩两件三件套与 DSL lint（评审回执承诺正式版后一周内）。
