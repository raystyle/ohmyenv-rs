//! 集成测试共享设施（S002 测试三件套之一）：ome 二进制运行 helper 与 expected 文件 oracle 断言。
//! `tests/common/mod.rs` 是 cargo 约定的共享模块，不会被当成独立测试目标（R004 一）。

use std::path::{Path, PathBuf};
use std::process::Output;

/// 夹具 catalog 路径（三种下载来源形态各一，字段契约见 R001）。
pub fn fixture_catalog() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tools.toml")
}

/// 跑 ome 二进制：注入 OME_CATALOG 指向夹具，外加调用方给的环境变量，返回完整 Output。
pub fn run_ome(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = assert_cmd::Command::cargo_bin("ome").expect("ome 二进制应已构建");
    cmd.env("OME_CATALOG", fixture_catalog());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.args(args);
    cmd.output().expect("ome 应可运行")
}

/// 归一化：CRLF 归 LF、沙盒临时路径替换为 <SANDBOX>、路径分隔符 `\` 归 `/`。
/// 分隔符归一是跨平台单 oracle 的取舍（路径内容照常比对）；若将来出现实质跨平台输出差异，
/// 再按 R004 启用 `.windows` 双 oracle。
fn normalize(text: &str, sandbox: Option<&Path>) -> String {
    let mut s = text.replace("\r\n", "\n");
    if let Some(root) = sandbox {
        s = s.replace(&root.display().to_string(), "<SANDBOX>");
    }
    s.replace('\\', "/").trim_end().to_string()
}

/// expected 文件 oracle：读 `tests/expected/<name>` 与 stdout 全量比对。
/// 黄金文件内以 `##` 起首的行为来源注释，比对前剥除（ome 自身组标题是 `# ` 单行，不冲突）。
pub fn assert_stdout_eq_golden(name: &str, out: &Output, sandbox: Option<&Path>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/expected")
        .join(name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读黄金文件失败: {}: {e}", path.display()));
    let expected = raw
        .lines()
        .filter(|l| !l.starts_with("##"))
        .collect::<Vec<_>>()
        .join("\n");
    let actual = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        normalize(&actual, sandbox),
        normalize(&expected, sandbox),
        "stdout 应与黄金文件 tests/expected/{name} 全量一致（## 注释行与沙盒路径已归一化）"
    );
}
