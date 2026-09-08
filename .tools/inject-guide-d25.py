# .tools/inject-guide-d25.py —— D25 一次性内容注入：catalog 各节插入 desc/guide_* 字段（幂等：已有 desc 跳过）
import io

p = r'catalog/tools.toml'
s = io.open(p, encoding='utf-8').read()
BS = chr(92)

def q(v):
    return '"' + v.replace('\\', BS * 2).replace('"', "'") + '"'

def arr(items):
    return '[' + ', '.join(q(i) for i in items) + ']'

def notes(lines):
    esc = [l.replace('\\', BS * 2) for l in lines]
    return '"""\n' + '\n'.join(esc) + '\n"""'

G = {
    'claude': ('终端智能体 Claude Code（claude 交互会话、claude -p 非交互）', [], ['%USERPROFILE%' + BS + '.claude'],
        ['升级走 claude 自身更新；ome install 对 PATH 在位的 agent 跳过（--force 才装进 EnvRoot）',
         '配置、hook 与凭据归 ohmyagents 域，ome 只管二进制']),
    'codex': ('OpenAI Codex CLI 智能体（codex exec 非交互）', [], ['%LOCALAPPDATA%' + BS + 'Programs' + BS + 'OpenAI' + BS + 'Codex'],
        ['升级走自更新；存量 PATH 在位即跳过纳管']),
    'grok': ('xAI Grok CLI 智能体', [], ['%USERPROFILE%' + BS + '.grok'],
        ['升级走自更新；存量 PATH 在位即跳过纳管']),
    'kimi': ('Kimi Code CLI 智能体', [], ['%USERPROFILE%' + BS + '.kimi-code'],
        ['升级走自更新；存量 PATH 在位即跳过纳管']),
    'ome': ('本环境管理器自身：doctor/install/status 三原语的部署管理 CLI', ['OHMYENV_ROOT', 'OME_MIRROR'],
        ['%LOCALAPPDATA%' + BS + 'Programs' + BS + 'ome', '%LOCALAPPDATA%' + BS + 'ohmyenv'],
        ['升级走 ome self update（dev/stable/git 三通道；官方失败回落镜像边车，OME_MIRROR=1 镜像优先）',
         '数据目录（catalog 副本与 SKILL.md）在 %LOCALAPPDATA%' + BS + 'ohmyenv；EnvRoot 默认 D:' + BS + 'ohmyenv（win）、~/.local/share/ohmyenv（POSIX）']),
    'herdr': ('终端 workspace/tab/pane 多路复用器，多 agent 并行会话的宿主', [], ['%APPDATA%' + BS + 'herdr'],
        ['服务器为常驻进程：升级后须面板外重启才换二进制（运行中 exe 锁定）',
         'socket、配置与日志在 %APPDATA%' + BS + 'herdr；conpty 运行态在装位目录内']),
    'pwsh': ('PowerShell 7 运行时', ['POWERSHELL_TELEMETRY_OPTOUT', 'POWERSHELL_UPDATECHECK'],
        ['%PROGRAMFILES%' + BS + 'PowerShell' + BS + '7'], ['遥测关闭由 install 写两个用户环境变量']),
    'wsl': ('Windows Subsystem for Linux 运行时（msi 安装型）', [], ['%LOCALAPPDATA%' + BS + 'wsl'],
        ['版本探测四段号归一三段对齐 tag；WSL 内环境按 R010 独立管理']),
    'docker': ('Docker Engine 容器运行时（服务加 compose 插件）', [], ['%PROGRAMDATA%' + BS + 'docker'],
        ['服务注册、daemon.json 与 compose 插件由 install 配置面写入；PATH 机器级；安装需管理员']),
    'dotnet': ('.NET SDK 运行时', ['DOTNET_CLI_TELEMETRY_OPTOUT'], ['%PROGRAMFILES%' + BS + 'dotnet'], []),
    'bun': ('Bun JavaScript 运行时与包管理器', [], ['%USERPROFILE%' + BS + '.bun'],
        ['同目录缺 bunx.exe 时 install 顺带补 shim']),
    'python': ('CPython 运行时', [], ['D:' + BS + 'ohmyenv' + BS + 'python'],
        ['虚拟环境管理优先用 uv（见 fnm/uv 条目）；本条目管解释器本体']),
    'nushell': ('Nushell 结构化 shell', [], ['D:' + BS + 'ohmyenv' + BS + 'nushell', '%APPDATA%' + BS + 'nushell'], []),
    'fnm': ('Node.js 版本管理器', ['FNM_MULTISHELL_PATH'], ['%LOCALAPPDATA%' + BS + 'fnm_multishells', '%LOCALAPPDATA%' + BS + 'fnm'],
        ['npm 全局 bin 随 fnm node 版本目录走（无静态路径，探测走 PATH 现查）',
         'node>=22 的 CLI（如 browser-harness 的 bh）依赖本条目供给']),
    'uv': ('Python 包与工具管理器（uv run/pip/tool）', ['UV_TOOL_DIR', 'UV_TOOL_BIN_DIR', 'UV_CACHE_DIR'],
        ['D:' + BS + 'ohmyenv' + BS + 'uv-tools'],
        ['uv tool install 型工具的 shim 落 uv-tools' + BS + 'bin（uv-git 通道）']),
    'vsbuild': ('VS Build Tools（MSVC 工具链与 CMake 组件）', [], ['D:' + BS + 'ohmyenv' + BS + 'vsbuild'],
        ['PATH 写机器级（MSBuild 与 cl 目录）；安装需管理员（gsudo 自动提权）',
         'Windows SDK 不随组件（ISO 分离装 Windows Kits，不进 ome）', '版本探测走 MSBuild -version']),
    'rust': ('Rust 工具链（rustup 引导，stable 六周滚动）',
        ['RUSTUP_HOME', 'CARGO_HOME', 'RUSTUP_DIST_SERVER', 'RUSTUP_UPDATE_ROOT'],
        ['D:' + BS + 'ohmyenv' + BS + 'rustup', 'D:' + BS + 'ohmyenv' + BS + 'cargo'],
        ['无 pin 语义（evergreen）：rustup update stable 即更新',
         '四个用户环境变量把 RUSTUP_HOME/CARGO_HOME 重定位 EnvRoot；rsproxy 双镜像',
         'cargo sparse 镜像 config.toml 由 install 写入（内容一致不重写）']),
    'go': ('Go 工具链', ['GOPROXY', 'GOPATH'], ['D:' + BS + 'ohmyenv' + BS + 'go', '%USERPROFILE%' + BS + 'go'],
        ['GOPROXY 镜像由 install 配置面设置；go version 子命令探测']),
    'zig': ('Zig 编译器', [], ['D:' + BS + 'ohmyenv' + BS + 'zig'],
        ['版本目录不展平（zip-dir 布局）；zig version 子命令探测']),
    'rmux': ('多路终端复用器（uv-git 装型）', [], [],
        ['升级前先停守护栈（--reload 与 kill-server）解锁 venv 再 install']),
    'openssh': ('OpenSSH 客户端（msi 安装型）', [], [],
        ['ssh -V 版本走 stderr（探测特判）；Windows OpenSSH 由 ome 无缝接管']),
    'age': ('age 文件加密（age/age-keygen）', [], ['%USERPROFILE%' + BS + '.config' + BS + 'age'],
        ['密钥文件属 secret-guard 域，ome 不触碰']),
    'sops': ('密文配置管理（加密 YAML/JSON 编辑）', ['SOPS_AGE_KEY_FILE'], ['%USERPROFILE%' + BS + '.config' + BS + 'sops'],
        ['密钥文件属 secret-guard 域，ome 不触碰']),
    'browser-harness': ('浏览器自动化平台（bh CLI：CDP 协议层加语义层、插件与 domain-skills）', [],
        ['%APPDATA%' + BS + 'npm'],
        ['npm-tgz 通道：tgz 过锚下载后 npm install -g；bin 名 bh（与工具名不同）',
         '需 node>=22 与 npm 在 PATH（fnm 供给）；探测走 PATH 现查（.cmd shim 经 cmd /c）']),
    'git': ('Git 版本控制', [], ['%USERPROFILE%' + BS + '.gitconfig'], []),
    'gh': ('GitHub CLI（PR、issue、gh api）', ['GH_TOKEN'], ['%APPDATA%' + BS + 'GitHub CLI'],
        ['GH_TOKEN 已设时只显在否不显值（凭据纪律）；限流时 resolve 回退 gh api 认证通道']),
    'aria2': ('多协议断点下载器', [], ['D:' + BS + 'ohmyenv' + BS + 'aria2'], []),
    '7z': ('7-Zip 压缩解压（含 7zr 引导器）', [], ['D:' + BS + 'ohmyenv' + BS + '7z'],
        ['版本探测走 --help 横幅（无 --version）']),
    'gsudo': ('Windows 提权执行器（sudo 等价）', [], ['D:' + BS + 'ohmyenv' + BS + 'gsudo'],
        ['vsbuild/docker 等需管理员的安装经它自动提权重跑']),
    'oscdimg': ('Windows ISO 镜像制作（ADK 工具）', [], [],
        ['无 --version：无参运行横幅含版本；Windows Kits 域工具']),
    'rg': ('ripgrep 代码与文本搜索', [], [], []),
    'jq': ('JSON 处理与查询', [], [], []),
    'mq': ('markdown 结构化查询（按节标题跨文件检索）', [], [], []),
    'yq': ('YAML 与多格式处理（jq 风格）', [], [], []),
    'starship': ('跨 shell 提示符主题', [], ['%USERPROFILE%' + BS + '.config'], []),
    'just': ('justfile 命令运行器（make 替代）', [], [], []),
    'ast-grep': ('结构化代码搜索与重写（AST 模式）', [], [], []),
    'rumdl': ('markdown lint 与自动修复（本项目门禁之一）', [], [], []),
    'shellcheck': ('shell 脚本静态检查', [], [], ['仅 Linux 入册（Windows 空态）']),
    'zoxide': ('智能目录跳转（cd 替代）', ['_ZO_DATA_DIR'], [],
        ['zsh 集成经 sheldon 插件声明；WSL 二进制与 win 分列']),
    'sheldon': ('zsh 插件声明式管理器', ['SHELDON_DATA_DIR'], [],
        ['上游无 Windows 资产（Windows 空态，Linux/mac 入册）']),
    'ffmpeg': ('音视频编解码与转码工具集（ffmpeg/ffprobe/ffplay）', [], ['D:' + BS + 'ohmyenv' + BS + 'ffmpeg'],
        ['win 源 gyan essentials、linux 源 BtbN GPL；mac 上游仅 Intel 构建（ARM 空态）']),
    'rclone': ('云存储同步 CLI（R2/S3 直传、本地挂载）', ['RCLONE_CONFIG'], ['%USERPROFILE%' + BS + '.config' + BS + 'rclone'],
        ['ohmycloud 上传链依赖（S3 直传）；SHA256SUMS 统一清单锚']),
    'reader': ('Agent 原生文档阅读（PDF/markdown/图片/Office 文本层提取与检索）', [], ['D:' + BS + 'ohmyenv' + BS + 'reader'], []),
    'lightpanda': ('无头浏览器（CDP serve、fetch、MCP server、AI agent 子命令）', [],
        ['%USERPROFILE%' + BS + '.local' + BS + 'bin'],
        ['win 空态：上游无 Windows 构建；WSL/mac 裸二进制',
         'version 子命令探测（--version 是 UnknownCommand）',
         'releases/latest 被 nightly 滚动 tag 占据：升级须 ome update lightpanda --tag <版本>']),
}

count = 0
for name, (desc, env, dirs, note_lines) in G.items():
    header = '[tools.' + name + ']'
    idx = s.find(header)
    if idx < 0:
        print('MISSING SECTION:', name)
        continue
    line_end = s.index('\n', idx) + 1
    # 幂等：该节已有 desc 则跳过
    nxt = s.find('[tools.', line_end)
    section = s[line_end: nxt if nxt > 0 else len(s)]
    if '\ndesc = ' in '\n' + section:
        continue
    block = ['desc = ' + q(desc)]
    if env:
        block.append('guide_env = ' + arr(env))
    if dirs:
        block.append('guide_dirs = ' + arr(dirs))
    if note_lines:
        block.append('guide_notes = ' + notes(note_lines))
    s = s[:line_end] + '\n'.join(block) + '\n' + s[line_end:]
    count += 1

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('injected', count, 'tools')
