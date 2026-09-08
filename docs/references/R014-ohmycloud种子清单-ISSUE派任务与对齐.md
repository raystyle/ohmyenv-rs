# R014：ohmycloud 种子清单，ISSUE 派任务与对齐

> 角色：现役流程。ome 与兄弟仓 ohmycloud（资源分发基建，env.ohmygh.com）之间，**种子资源清单只走 GitHub ISSUE 派任务和对齐**。catalog 是唯一 pin 权威；ohmycloud 源仓只读、零改动。

## 一、通道

| 侧 | 做什么 |
| --- | --- |
| ome | `catalog\tools.toml` 为唯一清单源；用 `.tools\seed-inventory.py` 抽可入镜对象；向 `raystyle/ohmycloud` 开或回 ISSUE |
| ohmycloud | 按 ISSUE 种子 / 下架 / 刷新 latest 段；通报对象数与差集 |
| 对齐完成 | ome 侧 `OME_TEST_MIRROR=1` 断官方源回落验收 |

ISSUE 标签用 `coordination` 加 `distribution`。中文排版：短段、列表、留白可换行。

## 二、入镜规则

1. **可入镜**：该平台 `version` + `asset` + `sha256` 三键齐。路径 `https://env.ohmygh.com/<tool>/<version>/<asset>`，同名 `.sha256` 边车。
2. **无锚不入镜**：缺 sha 的对象写「待 ome 回填」不派种。
3. **evergreen**：`ome` / `rust` / `vsbuild` 走 `latest` 段，边车即锚（ohmycloud#4 先例）。
4. **下架**：catalog 删除的工具在 ISSUE 里点名请镜像删除，避免脏对象。
5. **信任锚**：安装校验同 catalog pin，镜像不做第二份 manifest。

## 三、派发步骤

1. 跑 `uv run --script .tools/seed-inventory.py` 得到当期快照。
2. 与上一份已关闭的 ohmycloud 种子 ISSUE 做差集：新增、下架、sha 待回填、latest 段保持。
3. **差集先对域面 HEAD**（M012）：拟补种已 200 则不派；拟下架已 404 则不派。文本差集滞后于镜面是常态。
4. 在 `raystyle/ohmycloud` 开新 ISSUE（或在仍开放的种子 ISSUE 追评），正文含契约、差集、请求、完整清单（可折叠）。
5. 本仓 diary 记 ISSUE 号；ohmycloud 通报后再跑镜像闸门，并复核通报中的出入。

## 四、本仓脚本

```powershell
uv run --script .tools/seed-inventory.py
uv run --script .tools/seed-inventory.py --json
```

退出码 0 成功、2 读 catalog 失败。
