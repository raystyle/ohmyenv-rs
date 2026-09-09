# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D28 入册清单化（toolver 探测面迁 catalog 字段加结构机检加 checklist，2026-09-09 立项）。

### 依据

两族坑升格（G004 同型坑两犯）：toolver 正则漏带（rclone D22、gitleaks D26 两犯，装后探测 `?` 短路直接 None）；同名族资产 digest 错配（lightpanda mac，M014，靠 seed.py plan 兜住）。2026-09-08 diary 候选，用户「继续」推进。

### 方案骨架

1. **数据迁字段**：toolver `version_args`/`version_pattern` 两 match 表自源码迁 catalog 每节 `probe_args`（数组，缺省 `["--version"]`）与 `probe_pattern`（正则，单引号字面串）。命名避开已占用的 `version_pattern`（R001：python 资产名提版语义）。
2. **结构机检**：`tests/catalog_lint.rs` 对真仓与 fixtures 同规则断言：凡 `exe`/`linux_exe`/`mac_exe` 任一在位必须有 `probe_pattern`；`probe_pattern` 可编译且含捕获组；sha 族字段在位必为 64 位 hex。入册漏带直接红灯，不再靠装后实测暴露。
3. **入册 checklist**：R001 增节（catalog 节字段清单化：pin 四键同 tag、sha 与官方 digest 逐字核验、probe 两字段、装后探测验证、pin 落库后跑 `seed.py --plan` 域面 diff，M014 正解升格为纪律）。
4. **回填**：`.tools/inject-probe-d28.py` 幂等注入 46 节（仿 inject-guide-d25.py：`category` 行前插）；fixtures 三节同步。

### 完成定义

- 源码 toolver 两表删除，探测参数与正则唯一权威在 catalog。
- 机检测试在 cargo test 常规面（无闸门，离线跑真仓文件）。
- R001 字段表两行与 checklist 节在档。

### 验收

- `cargo test` 全绿（含新机检）；黄金文件不破（pin/status 渲染不含 probe 字段，若破按既定流程重生成并核对差异面）。
- 门禁四件套绿；本机 `ome status` 抽查三工具探测正常（新构建二进制）。
