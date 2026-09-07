# M103-PATH与注册表-错误

> 用户 PATH 注册、profile 标记块、死链判定错误速查。

## 行级条目

| 编号 | 日期 | 状态 | 现象 | 根因 | 正确处理 |
| --- | --- | --- | --- | --- | --- |
| M009 | 2026-09-07 | 已修正 | Linux/mac `ome deploy all` 后 profile 里 ome PATH 块只剩最后一个目录，go/pwsh/zig 与 `~/.local/bin` 互覆盖 | `add_user_path` 删除整个 `# >>> ome PATH` 块再写一条 export | 块内多行 append/去重；`remove_user_path` 只删该目录，空块才撤标记 |
| M010 | 2026-09-07 | 已修正 | 默认 EnvRoot `D:\ohmyenv` 时 doctor 把 `D:\ohmyenv-rs` 仓库 PATH 当死链 | 死链判定 `starts_with` 字符串前缀 | 用 `Path::starts_with` 按路径分量比较 |

## 范围注记

- Windows HKCU 比较需展开 `%VAR%` 并去掉尾斜杠，与 `add_path_entry` 同一套归一。
