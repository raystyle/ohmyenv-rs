# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**——当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：承接 ohmypwsh 部署、验收、自愈完整迁移（对应 GOAL.md 登记日 2026-09-01）。完整差集盘点、数据缺口与逐里程碑验收口径见 ohmypwsh 仓库的 P0026 方案（部署迁移 ohmyenv-rs 差集盘点与分域路线）；进度见 `TODO.md`。

1. **立项批次（2026-09-01 已完成）**：新址 `D:\ohmyenv-rs` clone 与基线门禁（修 CRLF 检出敏感测试）；安装形态整改——自部署进用户程序目录（Windows `%LOCALAPPDATA%\Programs\ome`）、元数据统一用户数据目录（`dirs::data_local_dir()/ohmyenv`）、self-deploy 同步 catalog 并清理旧 PATH 残留、catalog 解析四级（OME_CATALOG、exe 相邻、cwd、用户数据目录）；旧 clone 清理。
2. **M0 数据主权与回流**：psd1 Pos 侧 linux pin 回流 `catalog\tools.toml`（唯一现存数据源，回流由 ohmypwsh 侧协同）；`import-catalog.ps1` 改「只校验不再生」防再吞字段；go/zig/shellcheck/kimi 补录；定仓库 catalog 与部署态副本的同步纪律（self-deploy 即同步，pin 回写落点）。
3. **M1 mac 域（2026-09-01 已完成）**：Tool 增 mac 专属字段族（`mac_*`，回退链 mac → linux → 通用）与 exe 双语义；mac 真机 `cargo build/test` 与 `ome deploy/status` 验证（R011 流程六项全绿）；mac 逐工具补录随 M0 数据到位分批推进。
4. **M2 远端二进制下发**：交叉编译 linux-musl 与 darwin-arm64 单二进制加 catalog，`ome package` 产部署包供 scp，远端跑 `ome install/deploy`。
5. **M3/M4 验收与自愈**：`ome verify`（维度族数据化、密钥惰性注入只调自部署的 sops/age）与 `ome heal`（heal-map 数据化嵌入），双跑对账零 diff 为验收。
6. **M5/M6**：远端通道调系统 ssh 复用 mesh 成果；agent 四件套部署执行器接入；配合 ohmypwsh 部署链退役验收。

## 完成定义

- 五端可用 `ome install/deploy/verify/heal` 完成部署、验收、自愈。
- `ome verify` 五端全 PASS 且与 verify-five-ends 双跑对账一致；与 ohmypwsh P0026 M6 验收口径同步达成。
- 每批次本仓门禁全绿：cargo test、fmt --check、clippy -D warnings、两个 md 扫描；交叉 check 在 Linux/mac host 补跑。
