# AGENTS.md

本文件是协作规则的**最高约束**，四段职责依次为：**项目定位**、**操作规则**、**意图路由**、**资源索引**。

## 一、项目定位

> 本项目的本质与边界。根为定位，下分本质、边界、管理对象、方案索引。

1. **本质**
   - Oh My Env 是本机跨平台环境部署管理 CLI（Windows / Linux / macOS 三端）：自 ohmypwsh 五端控制总台的 `ohmyenv.ps1` 剥离的 Rust 实现，负责 31 个工具的版本解析、下载、校验、解压、PATH 注册、pin 锁定、日常更新，一个标准、一个配置。

2. **边界**
   - 管理本机 Windows 与 Linux（WSL 为本期 Linux 开发/验证主机）；远端与五端域（check / heal / omp / ssh-mesh）留在 ohmypwsh，不搬不碰。
   - 智能体（codex / claude / grok）的安装不属 ome 管理域，归 ohmyagents / ohmypwsh（2026-08-31 裁决）。
   - Linux 与 macOS 采用系统标准目录策略（如 `~/.local/share/ohmyenv`、`~/.local/bin`），不进 Windows 的 `D:\ohmyenv` 目录；mac 已于 2026-09-01 接管为开发主机与被管理端（M1，流程见 R011）。
   - 只读 ohmypwsh 与 ohmyagents：数据从它们的配置转换而来，源仓库零改动。

3. **管理对象**
   - 31 个工具的名录（`catalog\tools.toml` 是唯一 pin 源与静态字段权威，模式见 R001；M0 起数据主权在 ome，psd1 Pos 侧已一次性回流并冻结）。
   - EnvRoot：Windows 为 `D:\ohmyenv`；Linux 为 `~/.local/share/ohmyenv`（可经 `--env-root` / `OHMYENV_ROOT` 覆盖）。
   - 用户 PATH：Windows 为注册表 `HKCU\Environment\Path`；Linux 为当前 shell profile（如 `~/.bashrc`）或 `~/.config/ohmyenv/env.sh`（具体策略见 R010）。

4. **方案索引**
   - 数据模式：`docs\references\R001-catalog数据模式-tools-toml字段与pin语义.md`
   - 项目简介与命令：`README.md`
   - 研究：`docs\research\`（文件名即标题，按关键词搜）

## 二、操作规则

> 两类场景：**工作节奏**（何时做什么）与**写作编码**（写什么按什么标准）。每条下分可以与禁止。产品行为约束不在此层，见意图路由与 `docs\research\`。

### 工作节奏

1. **每轮对话**
   - 可以：先核对三原语 `GOAL.md`、`TODO.md`、`PLAN.md`；实质推进当场更新 todo 与 plan。
   - 禁止：不核对三原语就干活；偏离当前目标；推进了不更新 todo/plan。

2. **踩坑时**
   - 可以：当场按当前最大号接编 MNNN，落 `docs\mistakes\` 对应分类文件一行（文件名即错误主题，分类表见 `INDEX.md`）；同根因或同型坑合并聚合进已有条目（保留最早编号与首踩日期），不必每踩必新增；主题深挖落 `docs\research\`。
   - 禁止：只留在对话里反复试错。

3. **发现问题时**
   - 可以：任何问题都参照现有文档逻辑、结构与 `INDEX.md` 自修正，走五步闭环（循环自迭代）——**定位**：先搜 INDEX 与相关文档，确认是否已有规则、研究或参考覆盖；**归类**：文档错修文档、规则缺补规则（AGENTS 或对应细则）、知识缺落研究（六态）、出错模式记 mistakes、验证过的做法沉淀 references；**修正**：改在源头，下游引用、索引与三原语同步；**验证**：`rumdl check .` 加 `.tools` 两个 md 扫描（`md-ref-scan.py` 断链、`md-heading-scan.py` 标题括号），涉及结构再对账 INDEX 与磁盘；**提交**：一事一提交，diary 记钩子。
   - 禁止：跳过定位直接改（重复造已有规则）；只修表象不回写体系；问题只留在对话或记忆里；修完不跑验证；把临时补丁当最终方案不归档。

4. **交付变更时**
   - 可以：改代码同步对应文档，改文档同步索引与 `docs\diary\`；遵守命名标准；技术文档按文档标准细则写。
   - 禁止：只改代码不落文档；改了文档不更新索引。

5. **经验沉淀时（强规则，G004）**
   - 可以：成功的 plan 沉淀归 `docs\proven\`（方案与过程）；研究被实证后的做法与多次错误后沉淀成的正确工作流进 `docs\references\` 并挂意图路由或 INDEX。
   - 错误经验踩坑当场记 `docs\mistakes\`（同根因聚合）；同型坑二犯以上把正确处理升格成 references 工作流并互指。
   - 禁止：`[经验]` 断言只留在研究文档不落 references（检索不到等于没沉淀）；错误只记现象不记根因与处理；`[推断]`/`[假设]` 跳级进 references；一条知识两个权威落位互相重复。

6. **提交时**
   - 可以：`feat:` / `docs:` / `fix:` / `chore:` 前缀加中文描述；一次提交只做一件事。
   - 禁止：多事混一提交；未经指示推远端。

### 写作编码

7. **执行命令与写文件时**
   - 可以：Windows 命令用 PowerShell 7（`pwsh`），Linux / macOS / WSL 用该平台常规 shell；Markdown / Rust 源码 UTF-8；Windows 上需兼容 5.1 的脚本用 UTF-8 BOM。
   - 禁止：Windows 上默认用 `powershell.exe` 5.1；无 BOM 的中文 ps1 给 5.1 读。

8. **写 Rust 时**
   - 可以：先查 crates.io / docs.rs / GitHub 上是否已有最流行、最稳定、或已经覆盖本需求的库，检索走双通道细则 `docs\references\R005-选型研究细则-cratesio与github双通道.md`（crates.io 稳度四信号 + gh 流行活跃分辨，结论附证据）；选定后用最少代码接上，优先组合而不是自写协议、解压、HTTP、哈希、CLI 解析。
   - 禁止：在现成库已能稳定完成的前提下从零实现；为风格引入冷门或实验 crate；一次拉一堆用不上的依赖。

9. **写文档时**
   - 可以：遵守 `docs\guide\G001-文档标准细则-命名写作规范与rumdl检查.md`（树形、标题干净、无 emoji 与箭头等装饰符号、文件名即标题、rumdl）。
   - 禁止：标题带括号、口号或破折号（解释放标题下一行引用 `>`）；整段混杂不成树。

10. **写研究与测试文档时**
    - 可以：事实性断言必须标六态之一——`[实证]`（本机实测）、`[推断]`（逻辑推出）、`[经验]`（历史惯例）、`[记忆]`（待复核）、`[假设]`（待验证）、`[直觉]`（主观倾向）；标准见 `docs\guide\G002-研究标准细则-结构与六态标记.md`；研究与测试的结论断言不标六态即视为未完成。
    - 禁止：把「没验证」写成「已验证」（实证滥用）；断言不标六态；用猜测冒充结论。

11. **写测试时**
    - 可以：遵守 `docs\references\R004-测试标准细则-分层断言与门禁流程.md`。分层按官方三层（单元 `#[cfg(test)]`、集成 `tests\*.rs`、doctest），集成优先于单元；意图对应方法（冒烟断退出码、回归用黄金文件、验收对照 oracle）；测试名写成可读规格，负例带 `dies_` 前缀；期望值必须来自独立来源（规范、黄金文件、属性），断言只写稳定字段（标记行、退出码），放过 pid 与时间戳；测试体用 `TestResult` 加 `?` 传播错误；依赖真实 `D:\ohmyenv` 环境与注册表的测试按 `OME_TEST_REAL` 闸门 skip；测试设施收 `test-util` feature。
    - 禁止：重言式断言（期望值来自被测同款逻辑或镜像实现分支，AI 生成测试高发）；公开 API 测试塞 `mod tests{}` 不进 `tests\`；默认 mock（ome 拿临时 EnvRoot 沙盒，真机对齐走闸门）；计时进断言；测试设施无 feature gate 进生产构建；只测 happy path。

12. **写临时脚本时**
    - 可以：按需自定义的 ps1 / py / Rust 工具，有复用价值即归档 `.tools\`（规则与清单见 `.tools\README.md`；Python 带 PEP 723 头，用 `uv run --script` 运行，py 选库走 `docs\references\R008-项目工具Python库选型细则-pypi与uv.md`；PowerShell 选模块走 `docs\references\R009`）；文档结构大改（改名、编号、移目录）后跑 `uv run --script .tools\md-ref-scan.py` 做断链回归。
    - 禁止：可复用脚本散落仓库根或只留在对话里；把 `pypi.org/search` 或抓网页当可编程选型接口；用 sed 批改中文与反斜杠路径（用 `md-replace.py`）；归档不带自述与用法。

## 三、意图路由

> 需求意图与操作方法的映射。八命令细则见 `README.md` 与 `PLAN.md`。
> 显示名 Oh My Env；仓库独立维护在 `D:\ohmyenv-rs`（github.com/raystyle/ohmyenv-rs）；CLI 二进制 `ome`。EnvRoot 是 `D:\ohmyenv`（只放被管理工具，不放 ome 自身）。

- **查版本**：`ome query [tool|all] [--latest|--tag|--version]`（只解析版本与资产，不下载）
- **装工具**：`ome install [tool|all]`（装入 EnvRoot，不改 PATH）
- **部署**：`ome deploy [tool|all]`（安装 + 注册用户 PATH，默认锁定版本）
- **更新**：`ome update [tool|all]`（更新到最新版并锁定）
- **锁定**：`ome pin [tool|all] [--latest|--version]`（查看/设置 pin，lock 为别名）
- **看状态**：`ome status`（锁定 vs 已安装 vs PATH 三态对照）
- **日常更新**：`ome daily [--dry-run] [--include-breaking]`（同主版本自动、跨主版本保留、退出码 2）
- **自部署**：`ome init（self-deploy 兼容别名）`（复制二进制到用户程序目录 `%LOCALAPPDATA%\Programs\ome`、同步 catalog 到 `%LOCALAPPDATA%\ohmyenv` 并注册用户 PATH；Linux/mac 为 `~/.local/bin`）
- **查文档**：先搜 `INDEX.md` 定位编号，再读文件；rg / mq / ast-grep 全套搜索方法见四、资源索引
- **项目工具**：`.tools\`（自定义脚本归档；Python 用 `uv run --script .tools\<名>.py`，清单见 `.tools\README.md`；py 选库细则 `docs\references\R008`）；文档验证三件套：断链回归 `md-ref-scan.py`、标题括号 `md-heading-scan.py`、`rumdl check .`

八命令均在开发中，未全绿前禁止假装已经可跑；真机行为以 `ohmyenv.ps1` 为对照基准。

## 四、资源索引

> 定位看 `INDEX.md`（项目根目录，唯一索引：编号表、目录结构、代码文件位置）。本节是**配合 INDEX 的搜索与分析方法**。

**速记**：前缀定位 `P`（proven 归档）/ `S`（research 研究）/ `R`（references 开发测试参考）/ `G`（guide 元规范）/ `M`（mistakes 错误；文件 M1xx、行级 M0xx）；根目录三原语 `GOAL` / `PLAN` / `TODO`。

**搜索方法（文档）**：

```powershell
rg -n "关键词" INDEX.md                        # 1 先搜总索引，定位编号或文件
rg --files docs | rg 关键词                     # 2 按文件名搜文档
rg -n "关键词" docs\research docs\references    # 3 全文搜研究参考
rg -n "关键词" docs\mistakes\                   # 4 搜错误处理

# mq（markdown 结构查询，D:\ohmyenv\mq\mq.exe，jq 风格；section 模块必须 -A）
mq -F grep '.h2' docs\research\*.md             # 跨文件按节标题定位（文件:行号:标题）
mq -A 'section::section(., "关键结论")' 文档     # 抽整节内容（含正文）
mq -A -F json '.h1' 文档                        # 结构化 JSON（类型/深度/位置）
```

**搜索方法（代码）**：

```powershell
ast-grep outline -l rs --json src\              # 模块符号表（INDEX 代码表配符号清单）
ast-grep run -p 'pub fn name($$$) $$$' -l rs    # 按名定位定义，免疫注释与调用行
ast-grep run -p 'fn $NAME($$$) -> Result<$RET, String> $$$' -l rs --json  # 签名表
```

坑速查：mq 的 `.h.1` 是层级值不是文本（节点用 `.h`/`.h1`）；无 `.s` 选择器（用 section 模块）；ast-grep 的 fn 模式必须带 body 通配 `$$$`、可见性要写进模式、JSON 变量取 `metaVariables.single.<VAR>.text`（经验自 ohmyagents M107）。

**分析路径**：改产品行为先读 `docs\references\R001`（数据模式）再回 `README.md` 与 `PLAN.md`（命令语义）；踩坑查 `docs\mistakes\M1xx`；写码选库走 R005；测试规范 R004；新想法走 G003 五步；定位代码先 INDEX 模块表再 ast-grep 符号；抽文档节用 mq section。
