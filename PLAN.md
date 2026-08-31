# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**——当前目标怎么推进（步骤/标准/验收），随目标变化更新。
> 与 `TODO.md` 分工：todo = 做到哪；本文件 = 怎么做。

## 当前目标实施计划

> 当前目标（对应 GOAL.md 登记日 2026-08-31），任务项见 `TODO.md`。

1. 数据层：`.tools\import-catalog.ps1` 把 ohmypwsh `scripts\catalog.psd1`（Win 侧）+ `helpers.ps1` New-ToolDef 静态元数据转成 `catalog\tools.toml`（模式见 `docs\references\R001`），29 个工具、顺序同 ToolNames。psd1 只读不写。
2. 核心链路：catalog（toml_edit 读写 + pin 回写保序）、resolve（GitHub REST / cdn_url 模板 / cdn_index_url 三分支 + gh api 回退）、download（cache 复用 + ureq + curl 回退）、checksum（pin sha > 官方校验源三型）。先打通 query/pin。
3. 安装链路：extract 九分派（zip/targz/copy/gsudo/7z-extra/7zsfx/msi/rmux/single）、envpath（HKCU\Environment\Path，REG_EXPAND_SZ 保留、展开去重、前置）、toolver（每工具版本探测正则表，移植 helpers.ps1:888-935）、install 主编排（幂等跳过、EnvRoot 防穿越、全量重建、装后验版本、成功才回写 lock）。打通 install/deploy/update。
4. 收尾命令：status（locked/installed/path 三态）、daily（同主版本自动、跨主版本保留、exit 2、日志 logs\update-daily.log）、self-deploy（复制当前 exe 到 `D:\ohmyenv\ome\bin\ome.exe` 并注册用户 PATH；Windows 策略，Linux/mac 不进 ohmyenv 目录，留待后续）。
5. 测试：`tests\cli.rs` 临时 EnvRoot 沙盒（pin/status/幂等/防穿越负例）；真机对齐闸门测试对照 ohmyenv.ps1。
6. 验收：cargo build/test 全绿；`ome status` 与 `ohmyenv.ps1 status` 逐项一致；`ome query gh --latest` 同 tag；`ome daily --dry-run` 同判定；沙盒全链路 install 一个 zip 工具实测。

## 完成定义

- 八命令在真实 `D:\ohmyenv` 上行为与 ohmyenv.ps1 对齐（status 三态一致、query 同版本、daily 同判定）。
- `ome self-deploy` 后新终端 `ome` 可直接调用。
- ohmypwsh 仓库零改动。
