# R013-Agent友好IO契约-输出格式退出码与冻结面

> ome 对外输出契约的唯一权威（自 README 收敛而来，2026-09-07 用户裁定 README 只留介绍、部署使用与软件清单）。
> 吸收自 incurs 研究（S001）与 gh/git 实证（S003 三格式渲染、结构化错误、字段序稳定）。

## 一、输出三格式

全局 `--format kv|json|jsonl`（默认 kv），`--json` 为 json 简写：

- **kv**：`key=value` 逐行，块间空行，`#` 注释行为分组标题（可滤）；
- **json**：整批输出一个 JSON 数组文档，stdout 恒为合法 JSON（无数据为 `[]`）；
- **jsonl**：每块一行 JSON 对象，逐工具/逐维度即出（流式与结构化兼得）。

值一律字符串，字段序与 kv 行序一致（serde_json preserve_order）。

## 二、数据与错误分流

- 数据只走 stdout；进度与提示（`[INFO]/[OK]/[WARN]`）走 stderr；
- 错误出口为 OmeError（code/message/hint/exit_code），main 按 exit_code 退出；
  结构化模式下错误以单行 JSON `{"code","message","hint"?}` 附 stderr 末行，退出码不变形。

## 三、命令数据块字段

| 命令 | 数据块字段 |
| --- | --- |
| `query` | tool, tag, version, asset, size, url, sha256 |
| `pin` | tool, tag, version, asset, sha256 |
| `install` / `deploy` / `update` | tool, action, version, dir |
| `status` | tool, locked, installed, path, exe |
| `daily` | tool, action, from, to |
| `init` | action, exe, bin_dir, catalog, path |
| `package` | tool, version, package_dir, bin_dir, main_bin |
| `verify` | name, verdict |
| `heal` | dim, action, params, result, detail |
| `doctor` | check, status, detail（三层节另出 sys.* / agent / dep 块，D07） |

## 四、退出码

| 码 | 语义 |
| --- | --- |
| 0 | 成功 |
| 1 | 失败（verify/doctor 有 FAIL 项、heal 有 fail/partial、安装出错） |
| 2 | daily 有跨主版本保留项 |

## 五、对外冻结契约

> issue #4，2026-09-02 冻结。

`query` 与 `status` 的 `--format json` 字段集与退出码（0/1）为对外契约，ohmypwsh 镜像链
（build-wsl-image、download-posix-dev 替代）按此消费。契约演进只做**增量字段**（消费方按名
取值不受影响），删除或改名视为 breaking，需在提交与 diary 显式标注并通知消费方。
query 的 `sha256` 字段语义：解析 tag 与资产同 pin 时给锁定 sha256（未回填为空串），否则空串。
