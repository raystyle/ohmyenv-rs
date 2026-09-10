# AGENTS.md

本文件是协作规则的**最高约束**，四段职责依次为：**项目定位**、**工作规则**、**意图路由**、**资源索引**。只留规则骨架与指向，细则唯一权威在对应 G/R 文档（摘要层铁律：双份并行必漂移）。

## 一、项目定位

1. **本质**：Oh My Env（CLI 名 `ome`）是本机跨平台环境部署管理 CLI（Windows / Linux / macOS），独立仓库。负责 47 个工具（37 加 agent 四家与 zoxide/sheldon/ffmpeg/rclone/browser-harness/reader/lightpanda/gitleaks/typst 减 vault，含 ome 自管条目）的版本解析、下载、校验、解压、PATH 注册、pin 锁定、更新与 doctor 诊断（系统 / 依赖两层，D30 收窄）。一个标准、一个配置。成功标准：命令在部署系统上功能完整。下载官方失败回落兄弟仓 ohmycloud 的 env.ohmygh.com 镜像。
2. **边界**：只管本机（落在哪台机器就管哪台：Windows / Linux / macOS）。不做远程编排与五端总台。下载分发基建归兄弟仓 **ohmycloud**（域名 env.ohmygh.com；官方渠道失败回落该镜像，有 sha 锚（catalog pin 或 latest 段边车）才回落，D08）。跨仓协调（周知、回执与种子对账）一律走 herdr 会话同步，不再发 ISSUE（2026-09-10 裁；细则 R014；catalog 为唯一权威）。agent 四家（claude / codex / grok / kimi）**二进制安装**由 ome catalog 纳管（D07，PATH 在位即跳过）；agent 配置、hook、编排归 ohmyagents；omc agent deploy 已全面委托 ome（D29，细则 R014 六）。三活仓本地路径统一登记（重叠功能互相 review 直读对方仓代码，2026-09-10 裁）：ohmycloud = `D:\ohmycloud`、ohmyenv-rs = `D:\ohmyenv-rs`、ohmyagents-rs = `D:\ohmyagents-rs`。ohmycloud 与 ohmyagents 源仓只读、零改动。Linux 与 macOS 用系统标准目录策略，不进 `D:\ohmyenv`（细节 R010 / R011）。
3. **管理对象**：47 工具名录（`catalog\tools.toml` 唯一 pin 源与静态字段权威，九类 taxonomy 见 R001；agent 存量原地纳管）；EnvRoot（Windows `D:\ohmyenv`，Linux `~/.local/share/ohmyenv`，可经 `--env-root` / `OHMYENV_ROOT` 覆盖）；用户 PATH（Windows 注册表 `HKCU\Environment\Path`，POSIX 侧见 R010 / R011）。
4. **方案索引**：数据模式 R001；项目简介与命令 `README.md`；研究 `docs\research\`（文件名即标题）。

## 二、工作规则

### 工作节奏

1. **每轮对话**：先核对四原语（`PRD.md`、`GOAL.md`、`TODO.md`、`PLAN.md`）；实质推进当场更新。禁止不核对就干活、偏离当前目标、推进了不更新。
2. **踩坑时**：当场按当前最大号接编 MNNN 落 `docs\mistakes\` 一行；同根因同型坑合并进已有条目；深挖落 research。禁止只留在对话里反复试错。
3. **发现问题时**：走五步闭环（G003）：定位（先搜 INDEX）、归类（错修文档、缺补规则、知识落研究、出错记 mistakes、实证进 references）、修正（改在源头，下游同步）、验证（`rumdl check .` 加 `.tools` 三扫描）、提交（一事一提交，diary 记钩子）。禁止跳过定位直接改、只修表象不回写体系、修完不跑验证。
4. **交付变更时**：改代码同步对应文档，改文档同步索引与 `docs\diary\`。禁止只改代码不落文档、改了文档不更新索引。
5. **经验沉淀（G004 强规则）**：成功 plan 归 `docs\proven\`；实证做法与多犯沉淀的正确工作流进 `docs\references\` 并挂路由或 INDEX；同型坑二犯以上升格 references 并互指。禁止 `[经验]` 断言只留研究不落 references、错误只记现象不记根因、`[推断]`/`[假设]` 跳级、一条知识两个权威落位。
6. **提交时**：`feat:` / `docs:` / `fix:` / `chore:` 前缀加中文描述；一次提交只做一件事；未经指示不推远端。分支为 GitHub Flow 单干变体（直推 main 为基线，2026-09-08 裁，细节 ROADMAP 开发流程节）：并行会话或危险大改开短命分支，验证后 squash 进 main 并删。

### 写作编码

7. **执行命令与写文件**：Windows 用 PowerShell 7（`pwsh`），Linux / macOS / WSL 用平台常规 shell；Markdown / Rust 源码 UTF-8；兼容 5.1 的 ps1 用 UTF-8 BOM。禁止默认 `powershell.exe` 5.1、无 BOM 中文 ps1 给 5.1 读。
8. **写 Rust**：先按 R005 双通道查 crates.io / GitHub 选最流行稳定库，最少代码接上，优先组合不自写协议、解压、HTTP、哈希、CLI 解析。禁止现成库能完成时从零实现、引入冷门实验 crate。
9. **写文档**：遵守 G001（树形、标题干净、文件名即标题、rumdl 与 `mdcharlint.py` 禁字机检，豁免区见 G001 二）。禁止标题带括号、口号或破折号；整段混杂不成树。
10. **写研究与测试文档**：事实性断言必标六态之一（G002）：`[实证]`、`[推断]`、`[经验]`、`[记忆]`、`[假设]`、`[直觉]`。禁止把「没验证」写成「已验证」、断言不标六态、猜测冒充结论。
11. **写测试**：遵守 R004。三层分层集成优先；冒烟断退出码、回归黄金文件、验收对照 oracle；期望值来自独立来源，断言只写稳定字段；`TestResult` 加 `?`；真实环境测试按 `OME_TEST_REAL` 闸门 skip；设施收 `test-util` feature。禁止重言式断言、测试塞 `mod tests{}`、默认 mock、计时进断言、只测 happy path。
12. **写临时脚本**：可复用脚本归 `.tools\`（Python PEP 723 头用 `uv run --script`，选库走 R008 / R009；结构大改跑 `md-ref-scan.py` 断链回归）。禁止脚本散落、网页当选型接口、sed 批改中文与反斜杠路径（用 `md-replace.py`）。

### 文档义务表

> 动作到必须对齐的文档；漏一件即流程缺口。

| 动作 | 时机 | 义务 |
| --- | --- | --- |
| 新需求与澄清 | 澄清完成 | PRD 登记与状态流转，禁止静默假设 |
| 目标立项 | 开工前 | GOAL 起点与锚点、PLAN、TODO；GOAL 回指 D 编号 |
| 选型与调研 | 研究完成 | S 文档（六态）+ INDEX 研究节 |
| 写改源码与配置 | 改动完成 | README 同步；行为基线变化同步 guide；版本级成果进 CHANGELOG |
| 写测试 | 新层或新面 | 测试规范 guide 同步；INDEX 测试行 |
| 写脚本 | 归档时 | `.tools\README.md` 清单行 |
| 踩坑 | 当场 | mistakes 接编一行；INDEX 错误节同步 |
| 方案达成 | 验收全绿 | proven 回填、GOAL 历史行、INDEX 归档节、TODO 残表清退留指针 |
| 每次提交 | 提交后 | diary 当天记钩子 |
| 发布 | tag 推送后 | CHANGELOG 封版、ROADMAP 阶段状态；herdr 会话知会 ohmycloud 同步 omc tool status 镜像锚（D29） |
| 文档结构变更 | 改名移目录后 | INDEX 同步；断链回归必跑 |

## 三、意图路由

> 需求意图与命令映射的摘要层；参数与语义全表见 `README.md` 与 `PLAN.md`。
> 功能原语口径（PRD D10/D15/D16）：doctor（检测诊断）、install（幂等安装：下载加 PATH/注册表/配置）、status（三态对照）三原语，
> 其余命令为派生面（query 解析前置、update 安装时变、pin 锚操作、
> verify/heal 断言与自愈组合、init/self 辅助）。
> 仓库 `D:\ohmyenv-rs`（github.com/raystyle/ohmyenv-rs）；EnvRoot `D:\ohmyenv`（只放被管理工具，不放 ome 自身）。

- **查版本**：`ome query`（省略则全量；只解析版本与资产，不下载）
- **装工具**：`ome install`（省略则全量；下载到 EnvRoot，并注册 PATH、写注册表与配置）
- **更新**：`ome update`（省略则全量；install 到最新并锁定）
- **锁定**：`ome pin`（省略则全量；lock 为别名）
- **看状态**：`ome status`（锁定 / 已安装 / PATH 三态对照）
- **自部署**：`ome init`（self-deploy 别名；二进制进用户程序目录、catalog 同步、注册 PATH）
- **查文档**：先搜 `INDEX.md` 定位再读；方法见四
- **查/刷软件清单**：`ome catalog`（status 看解析面、云端锚与签名态，sync 立即从云端刷新并过 minisign 签名校验；自动刷新按 `OME_CATALOG_TTL` 走，`OME_OFFLINE=1` 关；公钥内嵌二进制，私钥只在开发机与 CI 密钥库）
- **项目工具**：`.tools\`（清单 `.tools\README.md`）；门禁四件套：`md-ref-scan.py` 断链、`md-heading-scan.py` 标题、`mdcharlint.py` 禁字、`rumdl check .`

命令真机对照基准为本机 catalog 与 EnvRoot 部署态；禁止把开发中能力当已交付宣称。

## 四、资源索引

> 定位看 `INDEX.md`（唯一索引：编号表、目录结构、代码文件位置）。本节是配合 INDEX 的搜索与分析方法。

**速记**：`P` 归档 / `S` 研究 / `R` 参考 / `G` 元规范 / `M` 错误（M1xx 分类、M0xx 行级）；根四原语 `PRD` / `GOAL` / `PLAN` / `TODO`。

**搜索方法（文档）**：

```powershell
rg -n "关键词" INDEX.md                        # 1 先搜总索引
rg --files docs | rg 关键词                     # 2 按文件名搜
rg -n "关键词" docs\research docs\references    # 3 全文搜研究参考
rg -n "关键词" docs\mistakes\                   # 4 搜错误处理

# mq（markdown 结构查询；section 必须带 -A）
mq -F grep '.h2' docs\research\*.md             # 按节标题跨文件定位
mq -A 'section::section(., "关键结论")' 文档     # 抽整节内容
```

**搜索方法（代码）**：

```powershell
ast-grep outline -l rs --json src\              # 模块符号表
ast-grep run -p 'pub fn name($$$) $$$' -l rs    # 按名定位定义
```

坑速查：mq 无 `.s` 选择器（用 section 模块）；ast-grep 的 fn 模式必须带 body 通配 `$$$`、可见性写进模式（ohmyagents M107）。

**分析路径**：改产品行为先读 R001 再回 `README.md` 与 `PLAN.md`；踩坑查 M1xx；选库走 R005；测试 R004；新想法走 G003 五步；定位代码先 INDEX 再 ast-grep；抽节用 mq section。
