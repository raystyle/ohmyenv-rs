# .tools/inject-probe-d28.py —— D28 一次性内容注入：catalog 各节插入 probe_pattern / probe_args
# 字段（幂等：节内已有 probe_pattern 则跳过）；插在 category 行前。
# 正则值逐条迁自 src/toolver.rs 原 version_pattern/version_args match 表（2026-09-09 删除）。
import io

P = {
    'claude': r'^(\d+\.\d+\.\d+)',
    'codex': r'codex-cli (\d+\.\d+\.\d+)',
    'grok': r'grok (\d+\.\d+\.\d+)',
    'kimi': r'^(\d+\.\d+\.\d+)',
    'ome': r'ome (\d+\.\d+\.\d+)',
    'herdr': r'^herdr\s+v?(\d+\.\d+\.\d+)',
    'pwsh': r'PowerShell\s+(\d+\.\d+\.\d+)',
    'wsl': r'(\d+\.\d+\.\d+)(?:\.\d+)?',
    'docker': r'Docker version (\d+\.\d+\.\d+)',
    'dotnet': r'^(\d+\.\d+\.\d+)',
    'bun': r'^v?(\d+\.\d+\.\d+)',
    'python': r'Python (\d+\.\d+\.\d+)',
    'nushell': r'^(\d+\.\d+\.\d+)',
    'fnm': r'fnm\s+v?(\d+\.\d+\.\d+)',
    'uv': r'uv (\d+\.\d+\.\d+)',
    'vsbuild': r'(\d+\.\d+\.\d+)',
    'rust': r'rustc (\d+\.\d+\.\d+)',
    'go': r'go version go(\d+\.\d+\.\d+)',
    'zig': r'(\d+\.\d+\.\d+)',
    'rmux': r'rmux\s+(\d+\.\d+\.\d+)',
    'openssh': r'OpenSSH_for_Windows_([\d.]+p\d+)',
    'age': r'^v?(\d+\.\d+\.\d+)',
    'sops': r'sops[ -]v?(\d+\.\d+\.\d+)',
    'browser-harness': r'^(\d+\.\d+\.\d+)',
    'git': r'git version (\S+)',
    'gh': r'gh version (\d+\.\d+\.\d+)',
    'aria2': r'aria2 version (\d+\.\d+\.\d+)',
    '7z': r'7-Zip[^\r\n]*?(\d+\.\d+)',
    'gsudo': r'gsudo\s+v?(\d+\.\d+\.\d+)',
    'oscdimg': r'OSCDIMG\s+(\d+\.\d+)',
    'rg': r'ripgrep (\d+\.\d+\.\d+)',
    'jq': r'jq-(\d+\.\d+\.\d+)',
    'mq': r'mq\s+v?(\d+\.\d+\.\d+)',
    'yq': r'version v?(\d+\.\d+\.\d+)',
    'starship': r'starship (\d+\.\d+\.\d+)',
    'just': r'just\s+v?(\d+\.\d+\.\d+)',
    'ast-grep': r'(\d+\.\d+\.\d+)',
    'rumdl': r'rumdl\s+(\d+\.\d+\.\d+)',
    'shellcheck': r'version:\s*(\d+\.\d+\.\d+)',
    'zoxide': r'zoxide (\d+\.\d+\.\d+)',
    'sheldon': r'sheldon (\d+\.\d+\.\d+)',
    'ffmpeg': r'ffmpeg version n?(\d+\.\d+(?:\.\d+)?)',
    'rclone': r'rclone\s+v(\d+\.\d+\.\d+)',
    'reader': r'reader\s+(\d+\.\d+\.\d+)',
    'lightpanda': r'^(\d+\.\d+\.\d+)',
    'gitleaks': r'gitleaks version (\d+\.\d+\.\d+)',
}

# 参数特例（其余缺省 ["--version"]）：oscdimg 无参读横幅；zig/go/lightpanda 子命令 version
ARGS = {
    '7z': ['--help'],
    'rmux': ['-V'],
    'oscdimg': [],
    'openssh': ['-V'],
    'vsbuild': ['-version'],
    'zig': ['version'],
    'go': ['version'],
    'lightpanda': ['version'],
}

p = 'catalog/tools.toml'
s = io.open(p, encoding='utf-8').read()

count = 0
for name, pattern in P.items():
    header = '[tools.' + name + ']'
    idx = s.find(header)
    if idx < 0:
        print('MISSING SECTION:', name)
        continue
    nxt = s.find('[tools.', idx + 1)
    section = s[idx: nxt if nxt > 0 else len(s)]
    if '\nprobe_pattern = ' in section:
        continue
    insert = s.index('\ncategory = ', idx) + 1
    block = ["probe_pattern = '" + pattern + "'"]
    if name in ARGS:
        args = ARGS[name]
        block.insert(0, 'probe_args = [' + ', '.join("'" + a + "'" for a in args) + ']')
    s = s[:insert] + '\n'.join(block) + '\n' + s[insert:]
    count += 1

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('injected', count, 'tools')
