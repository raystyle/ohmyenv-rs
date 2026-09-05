# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D07 doctor 核心化与 agent 入册（登记日 2026-09-05，追问链三轮六裁）。核心图景：doctor 升 ome 核心命令做三层诊断，检测驱动安装、幂等检测安装贯穿；agent 二进制安装域由 oma 反转回 ome 承载。

### 裁定边界

> 六裁原文口径。

1. oma 以后只管配置 agent、hook、编排，不管 agent 的升级和安装（D06 方向反转，其五端成果转过渡态）。
2. ome 承载：doctor 重构 + agent 入册 + install/deploy 幂等安装 agent 与依赖。
3. oma 的 agents install/update 与 `catalog\agents.toml` 迁册 ome（deprecated 标记不删，数据对接）。
4. 依赖六类终稿：agent、运行时、运行时管理器（uv/fnm）、编译器（rust/go/zig/vsbuild 含 C 编译）、cli 工具、运行时衍生。
5. 本批先 doctor；命令面大重构（子命令重排）另立项。
6. oma 登录态/hook 形态/状态栏/会话健康四类检查归 oma agents 域；doctor 的 agent 层只管二进制装好、版本正确、token 配置可用性，不取设置。

### 方案骨架

> 四切片，1 与 4 可并行起步。

1. **切片 1：catalog agent 条目建模**：oma `catalog\agents.toml` 四家 pin、sha、双渠道（github 主 CDN 兜底）、sums 模式（asset 清单/边车/manifest）迁 `catalog\tools.toml`；Tool 字段扩展 agent 特型（多渠道 sums、token 探测位）；taxonomy 演化草案（现七类加 agent 类为八类，uv/fnm 自运行时衍生挪运行时管理器类）**需用户过目终稿**（七类系 09-02 四轮定稿）；R001 同步数据模式。
2. **切片 2：doctor 三层重构**：系统层（OS、架构、指令集，吸收 oma caps 模块经验）；agent 层（四家二进制在位、版本对 pin、token 配置可用性探针，不取设置）；依赖层（六类清单逐项三态 + 环境错误检测：版本漂移、PATH 死链、探测失败，承接现 doctor 九项口径）；输出 kv/json 三格式与 Ok/Warn/Block 三态。
3. **切片 3：install/deploy 幂等覆盖 agent**：agent 条目接进 install/status/pin/daily/verify 既有链路；存量纳管语义（已装任何来源即跳过，对齐 oma agents install 判定）；oma 自管根 `~/.ohmyagents/agents` 布局的对齐或接管策略在切片内定。
4. **切片 4：oma 迁册配合与跨仓收口**：oma agents install/update 加 deprecated 提示指向 ome；oma `catalog\agents.toml` 头注记数据权威转 ome；ohmypwsh#9 口径更新（agent 节冻结的承接方为 ome）；五端 doctor 验收。

### 完成定义

- `ome doctor` 三层输出成型：系统层、agent 层（四家二进制/版本/token）、依赖层六类分组加环境错误检测，kv/json 双格式。
- `ome install [agent]` 幂等：存量跳过、二连跑零变更；status/pin/daily 对 agent 条目语义正确。
- taxonomy 八类终稿经用户过目；R001、INDEX、README 同步。
- oma 侧 deprecated 标记与数据源注记落位；ohmypwsh#9 口径更新；五端 doctor 实跑验收。
- 每批次本仓门禁全绿：cargo test、fmt --check、clippy -D warnings、md 四件套。
