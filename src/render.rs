//! render：单一渲染层（吸收自 incurs 的「handler 结构化产出 + 单一渲染层」模式，S003 扩展三格式）。
//! 纪律：stdout 只走数据，人称提示（[INFO]/[OK]/[WARN]/[HINT]）一律 stderr。
//! 命令产出统一收敛为 Vec<(key, value)> 块交本层输出，不在命令里散写 println!。
//!
//! 格式三态（`--format`，`--json` 为 json 简写）：
//! - kv（默认）：key=value 逐行，块间空行，分组走 `#` 注释行（header）；
//! - json：块累积为对象数组，命令收尾 finish 一次性输出（stdout 恒为合法 JSON 文档）；
//! - jsonl：每块立即输出一行 JSON 对象（流式与结构化兼得，status 逐工具即出）。
//!
//! 值一律字符串：kv 行直转 JSON 值，类型不伪装（gh 的字段类型化留待需要时再做）。

use std::cell::RefCell;

/// 输出格式。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    /// key=value 逐行（默认）
    Kv,
    /// 整批 JSON 数组，finish 收尾输出
    Json,
    /// 每块一行 JSON，立即输出
    Jsonl,
}

thread_local! {
    static FORMAT: RefCell<Format> = const { RefCell::new(Format::Kv) };
    static BLOCKS: RefCell<Vec<serde_json::Value>> = const { RefCell::new(Vec::new()) };
}

/// 设定输出格式（main 解析完全局 --format/--json 后调用一次）。
pub fn set_format(f: Format) {
    FORMAT.with(|m| *m.borrow_mut() = f);
}

/// 当前格式。
pub fn current_format() -> Format {
    FORMAT.with(|m| *m.borrow())
}

/// 是否结构化输出（json/jsonl）：错误出口据此切换 stderr 单行 JSON。
pub fn is_structured() -> bool {
    !matches!(current_format(), Format::Kv)
}

/// 把一组 key=value 格式化为文本块（纯函数，便于测试）。
pub fn format_rows(rows: &[(String, String)]) -> String {
    rows.iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 把一组 key=value 转为 JSON 对象（纯函数，便于测试；值一律字符串）。
pub fn block_value(rows: &[(String, String)]) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in rows {
        obj.insert(k.clone(), serde_json::Value::String(v.clone()));
    }
    serde_json::Value::Object(obj)
}

/// 输出一组 key=value 到 stdout：kv 逐行打印；jsonl 立即单行 JSON；json 累积待 finish。
pub fn emit(rows: &[(String, String)]) {
    if rows.is_empty() {
        return;
    }
    match current_format() {
        Format::Kv => {
            let text = format_rows(rows);
            if !text.is_empty() {
                println!("{text}");
            }
        }
        Format::Jsonl => {
            let line = serde_json::to_string(&block_value(rows)).unwrap_or_default();
            println!("{line}");
        }
        Format::Json => BLOCKS.with(|b| b.borrow_mut().push(block_value(rows))),
    }
}

/// json 模式收尾：把累积的块输出为一个 JSON 数组（空批输出 []，与 gh 空列表语义一致）。
/// kv/jsonl 模式无操作。main 的成功与错误出口都调用（错误前已产出的数据块照常上 stdout）。
pub fn finish() {
    if current_format() != Format::Json {
        return;
    }
    let blocks = BLOCKS.with(|b| std::mem::take(&mut *b.borrow_mut()));
    if let Ok(text) = serde_json::to_string_pretty(&blocks) {
        println!("{text}");
    }
}

/// 工具组分隔（多工具输出之间的空行；结构化模式下为空操作）。
pub fn blank() {
    if !is_structured() {
        println!();
    }
}

/// 组标题：以 `# ` 前缀注释行输出到 stdout（机器可滤，人可读分组）；结构化模式下为空操作。
pub fn header(text: &str) {
    if !is_structured() {
        println!("# {text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 渲染_kv逐行() {
        let rows = vec![
            ("tool".to_string(), "jq".to_string()),
            ("version".to_string(), "1.8.2".to_string()),
        ];
        assert_eq!(format_rows(&rows), "tool=jq\nversion=1.8.2");
    }

    #[test]
    fn 渲染_空组产空串() {
        assert_eq!(format_rows(&[]), "");
    }

    #[test]
    fn 渲染_块转json对象值全字符串() {
        let rows = vec![
            ("tool".to_string(), "gh".to_string()),
            ("count".to_string(), "3".to_string()),
        ];
        let v = block_value(&rows);
        let obj = v.as_object().expect("应为对象");
        assert_eq!(obj.get("tool"), Some(&serde_json::json!("gh")));
        // 数字也保持字符串（kv 行直转，类型不伪装）
        assert_eq!(obj.get("count"), Some(&serde_json::json!("3")));
    }

    #[test]
    fn 格式_默认kv且可切换() {
        assert_eq!(current_format(), Format::Kv);
        set_format(Format::Jsonl);
        assert!(is_structured());
        set_format(Format::Kv);
        assert!(!is_structured());
    }
}
