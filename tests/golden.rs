//! 黄金文件回归测试（S002 测试三件套之一）：expected 文件 oracle 全量比对 stdout。
//! 期望值纪律（R004）：黄金文件先跑程序取形态、人工核对契约后冻结，进 git 可审，不自动接受变更。

mod common;

use std::process::Output;

fn assert_ok(out: &Output) {
    assert!(
        out.status.success(),
        "应成功退出: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn pin_无选项_全量输出对齐黄金文件() {
    // 夹具三工具均已 pin：纯读路径，不触网不回写，输出确定
    let out = common::run_ome(&["pin"], &[]);
    assert_ok(&out);
    common::assert_stdout_eq_golden("pin.txt", &out, None);
}

#[test]
fn status_沙盒_全量输出对齐黄金文件() {
    // 沙盒 EnvRoot：exe 不存在 → installed=-；path 只读真实用户 PATH，沙盒 bin 不在其中 → false
    let dir = tempfile::tempdir().expect("创建沙盒失败");
    let env_root = dir.path().join("envroot");
    let root = env_root.to_string_lossy().to_string();
    let out = common::run_ome(&["status", "--env-root", &root], &[]);
    assert_ok(&out);
    // 输出含沙盒临时路径，比对前归一化为 <SANDBOX>（见 tests/common/mod.rs）
    common::assert_stdout_eq_golden("status.txt", &out, Some(&env_root));
}
