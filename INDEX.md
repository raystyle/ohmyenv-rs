# INDEX：唯一索引

> 全仓库唯一索引：文档编号表、目录结构、代码文件位置。查文档先搜本文件。

## 文档编号表

> 前缀：P=proven 方案归档 / S=research 研究 / R=references 开发参考 / G=guide 元规范 / M=mistakes 错误速查。

| 编号 | 文件 | 主题 |
| --- | --- | --- |
| G001 | `docs\guide\G001-文档标准细则-命名写作规范与rumdl检查.md` | 文档命名、写作规范与 rumdl 检查 |
| G002 | `docs\guide\G002-研究标准细则-结构与六态标记.md` | 研究文档结构与六态标记 |
| G003 | `docs\guide\G003-工作流标准细则-从登记到归档五步.md` | 想法从登记到归档五步工作流 |
| R001 | `docs\references\R001-catalog数据模式-tools-toml字段与pin语义.md` | tools.toml 字段模式与 pin 回写语义 |
| R004 | `docs\references\R004-测试标准细则-分层断言与门禁流程.md` | 测试分层断言与门禁（真机对齐闸门 OME_TEST_REAL） |
| R005 | `docs\references\R005-选型研究细则-cratesio与github双通道.md` | Rust 库与项目选型双通道 |
| R008 | `docs\references\R008-项目工具Python库选型细则-pypi与uv.md` | 项目工具 Python 选库与 uv |
| R009 | `docs\references\R009-项目工具PowerShell模块选型细则-psgallery与psresourceget.md` | 项目工具 PowerShell 模块选型 |
| R010 | `docs\references\R010-linux开发接管-环境准备与构建验证.md` | Linux 开发主机接管：工具链、构建验证、平台门控、两端分工 |
| S001 | `docs\research\S001-incurs选型研究-不迁移只吸收三模式.md` | incurs 框架选型裁决：不迁移，吸收错误结构、单一渲染层、帮助元数据三模式 |

不编号文档：`docs\guide\template.md`（方案模板）。

## 根目录文档

| 文件 | 角色 |
| --- | --- |
| `GOAL.md` | 任务目标管理（起点/锚点/进程/历史） |
| `PLAN.md` | 当前目标规划指导 |
| `TODO.md` | 当前目标任务进度清单 |
| `AGENTS.md` | 协作规则最高约束 |
| `README.md` | 项目简介与快速开始 |
| `CHANGELOG.md` | 版本里程碑 |

## 目录结构

| 目录 | 说明 |
| --- | --- |
| `src\` | Rust 源码，平铺模块（无子目录） |
| `catalog\` | `tools.toml` 唯一 pin 源（29 工具） |
| `tests\` | 集成测试（assert_cmd + predicates） |
| `.tools\` | 可复用脚本归档（清单见 `.tools\README.md`：import-catalog.ps1、md-ref-scan.py、md-heading-scan.py、md-replace.py） |
| `docs\` | proven/research/references/guide/mistakes/diary 六类 |
| `bin\` | self-deploy 产物（ome.exe，注册进用户 PATH；git 忽略） |

## 项目日记

> 位置 `docs\diary\`；一天一篇，当天总结与自省。

| 日期 | 文件 | 主题 |
| --- | --- | --- |
| 2026-08-31 | `docs\diary\2026-08-31-ome立项-文档体系与结构平移.md` | 立项、脚手架、catalog 转换、文档体系平移 |

## 错误速查分类

> 位置 `docs\mistakes\`；分类文件 M1xx，行级 M0xx。暂无条目；首踩时按下表接编分类并登记本节。

| 编号 | 文件 | 覆盖主题 | 行级条目 |
| --- | --- | --- | --- |
| M101 | 待建 | 版本解析与下载错误（REST、cdn、网络、哈希） | |
| M102 | 待建 | 解压与安装错误（九分派、防穿越、幂等） | |
| M103 | 待建 | PATH 与注册表错误（HKCU、展开、去重） | |
| M104 | 待建 | 文档与命名错误（命名、六态、diary、标题规范） | |
| M105 | 待建 | 工具链与脚本错误（sed、grep、PowerShell、中文路径） | |
| M106 | `docs\mistakes\M106-catalog转换与数据保真-错误.md` | catalog 转换与数据保真错误（转换合并规则、psd1 与 New-ToolDef 分歧） | M001 |

迭代规则：踩坑按当前最大号接编 MNNN 进对应分类文件（M0xx 行级、新分类用 M1xx 接编）；一行一事；同根因或同型坑**可合并聚合**进已有条目（保留最早编号与首踩日期，聚合后的正解写全）；反复踩落 `docs\research\`；改「正确处理」不删历史行；新分类文件登记本节。

## 代码文件位置

| 文件 | 职责 |
| --- | --- |
| `src\main.rs` | clap CLI 入口与子命令分派（八命令；输出纪律与示例元数据在文件顶部） |
| `src\omerr.rs` | 机器可读错误四元组（code/message/hint/exit_code），main 按 exit_code 退出 |
| `src\render.rs` | 单一渲染层：stdout 只走 key=value 数据，组标题走 # 注释行 |
| `src\catalog.rs` | tools.toml 读写、EnvRoot 解析、pin 回写 |
| `src\resolve.rs` | 版本解析三分支（GitHub REST / cdn 模板 / HashiCorp index） |
| `src\download.rs` | 资产下载与缓存复用 |
| `src\checksum.rs` | sha256 校验与官方校验源 |
| `src\install.rs` | 安装主编排（幂等、防穿越、验版本、回写） |
| `src\extract.rs` | 解压/安装九分派 |
| `src\envpath.rs` | 注册表用户 PATH 管理 |
| `src\toolver.rs` | 已装版本探测参数与正则表 |
| `src\status.rs` | status 三态对照与 daily 报告 |
| `src\selfdeploy.rs` | 自部署到 D:\ohmyenv\ome\bin |
