//! render：单一渲染层（吸收自 incurs 的「handler 结构化产出 + 单一渲染层」模式）。
//! 纪律：stdout 只走数据（key=value 逐行），人称提示（[INFO]/[OK]/[WARN]/[HINT]）一律 stderr。
//! 命令产出统一收敛为 Vec<(key, value)> 交本层输出，不在命令里散写 println!。

/// 把一组 key=value 格式化为文本块（纯函数，便于测试）。
pub fn format_rows(rows: &[(String, String)]) -> String {
    rows.iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 输出一组 key=value 到 stdout。
pub fn emit(rows: &[(String, String)]) {
    let text = format_rows(rows);
    if !text.is_empty() {
        println!("{text}");
    }
}

/// 工具组分隔（多工具输出之间的空行）。
pub fn blank() {
    println!();
}

/// 组标题：以 `# ` 前缀注释行输出到 stdout（机器可滤，人可读分组）。
pub fn header(text: &str) {
    println!("# {text}");
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
}
