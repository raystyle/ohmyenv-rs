import io

def rep(p, old, new, count=1):
    t = io.open(p, encoding='utf-8').read()
    assert old in t, (p, old[:50])
    io.open(p, 'w', encoding='utf-8', newline='\n').write(t.replace(old, new, count))
    print(p, 'ok')

# ── O3-b：fnm 装后 profile 钩子（交互用户面；非交互由 O3-a 治本）──
rep('src/install.rs',
 '''/// 子进程 PATH：npm 在 PATH 直用；否则 prepend fnm node bin（O3）。''',
 '''/// O3-b：fnm 装后写 profile 钩子（`eval "$(fnm env)"` 进 ome 标记块）。
/// 交互 shell 经钩子取 node；非交互（omc hostExec）由 fnm_node_bin 进程内解析治本，
/// 双面覆盖。幂等：块内已有 fnm env 行不重写（merge_env_exports 同 KEY upsert 语义）。
fn ensure_fnm_shell_hook(name: &str) {
    if name != "fnm" || cfg!(windows) {
        return;
    }
    if let Some(profile) = crate::platform::user_profile_path() {
        let text = std::fs::read_to_string(&profile).unwrap_or_default();
        let merged = crate::platform::merge_env_exports(&text, "FNM_ENV_HOOK", "$(fnm env)");
        // merge_env_exports 生成 export KEY="value" 形态；fnm 需要的是 eval 执行形态，
        // 借标记块机制写 eval 行：手写块内追加（幂等判据 = 已含该行）
        if !text.contains("eval \\"$(fnm env)\\"") {
            let _ = std::fs::write(&profile, format!("{text}\\n# >>> ome fnm >>>\\neval \\"$(fnm env)\\"\\n# <<< ome fnm <<<\\n"));
            eprintln!("[OK] fnm shell 钩子已写入 {}（新终端生效；node 需 fnm install）", profile.display());
        }
    }
}

/// 子进程 PATH：npm 在 PATH 直用；否则 prepend fnm node bin（O3）。''')
