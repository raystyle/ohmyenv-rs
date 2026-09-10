# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**：按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

D33 软件清单云端化与实时刷新（2026-09-10 用户令「开工」，按建议三项定稿）：
运行态 catalog 按 TTL 从云端刷新用户数据副本，新增 `ome catalog status/sync`，消费侧不依赖发版。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| D33：立项与口径定稿 | 已完成 | 用户方向「软件清单放 env 云端可实时更新，无需动 ome 即配置播种新软件」；三项定稿：权威落位（仓库为开发与离线兜底源加云端为运行态权威）、刷新时机（TTL 24h 自动加显式 `ome catalog status/sync`；`OME_CATALOG_TTL` 与 `OME_OFFLINE` 可关）、信任锚（镜像自算 sha256 边车，先边车后资产，签名留后）；PRD/GOAL/PLAN/TODO 落档 | 2026-09-10 |
| D33：刷新模块与命令面 | 已完成 | `src/catalogsync.rs`（边车锚刷新、`.last-sync` 标记 TTL、状态查询、纯函数五测）；CLI `ome catalog [status\|sync]`（render 数据块）；main 在解析后加载前按 TTL 自动刷新（仅用户数据副本，自动路径单次 5s 探活加退避标记）；download 层加 `fetch_text_short` 与 pub `parse_sidecar_sha` | 2026-09-10 |
| D33：测试与验收 | 已完成 | 单测五枚加 gated 真网测 `tests/catalog_refresh.rs`（锚一致、幂等 current、TTL 内 fresh）；真机 `ome catalog sync` 把部署副本刷到云端 CFF2B44A（含 typst），旧部署二进制 ome 0.2.1 在非仓库目录读到 typst 三态齐；`OME_CATALOG_TTL=0` 不刷新实证、人为漂移后自动还原实证 | 2026-09-10 |
| D33：文档同步与收口 | 已完成 | R013 命令面与数据块、README 示例与速查表、SKILL 命令图与语义要点、`--llms`（顺修 41 工具旧计数改 47）、AGENTS 意图路由、R001 四.7 升格与 checklist 八、CHANGELOG、diary 一篇；cargo test 全绿加 clippy 干净加四件套绿 | 2026-09-10 |
| D32：三平台锚核验 | 已完成 | release v0.15.1 资产清单抓取；win / linux / mac 三资产本机下载实测 sha256 与 GitHub digest 逐字一致（19CE3551 / A6D077D0 / 48F62ED0） | 2026-09-10 |
| D32：catalog 节入册 | 已完成 | `[tools.typst]` 追加 cli 类节尾：win zip 展平（单包裹层）、linux/mac tarxz-bin；probe `typst --version`；pin 四键同 tag v0.15.1；catalog 头注释 46 改 47 | 2026-09-10 |
| D32：计数与四原语同步 | 已完成 | 计数 47：AGENTS 两处、README 三处含类表、SKILL、INDEX、catalog 头注释；PRD D32、GOAL 锚点与时间线、PLAN 改档、CHANGELOG Unreleased、diary 一篇；顺修 INDEX 日记表 09-08 与 09-09 两行欠账 | 2026-09-10 |
| D32：门禁与真机验收 | 已完成 | cargo test 全绿（lib 90 加 cli 28 加 catalog_lint 2 加 golden 2 加 install 2 加 mirror 7 加 real 7）与 rumdl 加三扫描绿；win 首装 installed 加幂等二连 skipped、`ome status` 三态齐（0.15.1 加 path=true）；`seed.py --plan` 101 对象 97 synced 记 typst 三件 sidecar-missing | 2026-09-10 |
| D32：推送与镜像入镜验收 | 已完成 | 用户令「推」：五笔推送 main，CI build 三平台 job 全绿加 seed-mirror 绿（uploaded 4 含 typst 三件）；镜像三资产与边车 HEAD 200 且 sha 与 catalog pin 逐字一致；镜像 catalog 与仓库 catalog 同 sha（cff2b44a…）；本机 --plan 复跑 99 synced | 2026-09-10 |
| ome/stable 段 CI 直推补齐（ohmycloud 探 404） | 已完成 | mirror-r2 只挂 main 且只灌 dev/latest 是根因；seed.py 拆 --ome-dev/--ome-stable、build.yml v* tag 加 dispatch 双入口、selfupdate/doctor 切 ome/stable；dispatch 回灌 v0.2.0 纠正灾备手推态（win 锚漂移加 darwin 边车缺），三资产边车与 release digest 三方一致、dev 段未破 | 2026-09-10 |
| #10 余量：codex linux/mac 嵌套 bin 布局修复 | 已完成 | 布局实证（tar 内 bin/ 加 codex-path/ 加 codex-resources/）；POSIX 改装 `~/.local/share/codex` 注册其 bin（与 win 同构）；catalog 字段修复，catalog_lint 2/2 绿 | 2026-09-10 |
| #10 余量：catalog 裸端自举通道 | 已完成 | resolve_catalog_path 四级 miss 自动拉取（官方 raw 优先、镜像 ome/catalog 边车锚回落、解析验证防半截）；seed.py 路线 B 增推 catalog 本体加边车；ohmycloud 临时推 OME_CATALOG 方案可退役 | 2026-09-10 |
| D30：三仓定稿回执与 PRD 终稿 | 已完成 | omc 回执 2026-09-10：日期定 09-12（oma 同步改期，对齐版 oma v0.5.0）、削减清单无异议、推送确认、tag 后 ome/latest 拆 stable；PRD/GOAL/R014/ROADMAP 终稿落档 | 2026-09-10 |
| D30：doctor agent 层削减落码 | 已完成 | AgentHealth/agent_health/token_state 整体删除（五字段健康块与 token 凭据探测退出 ome）；main.rs 消费面改 collect_status；依赖层智能体依赖组保留；R013 契约、README 三处、SKILL、--llms、INDEX、AGENTS、selfdeploy 渲染器口径同步；lib 90 加 cli 28 加 catalog_lint 2 加 golden 2 加 install 2 加 mirror 7 加 real 7 全绿、clippy 干净 | 2026-09-10 |
| D30：v0.2.0 封版七步 | 已完成 | 急令当日全过：CHANGELOG 收口归 0.2.0（feac7d7）、ROADMAP 阶段七切进行中、tag v0.2.0 推送（GPG 无私钥降级 annotated，记 diary）、CI v* 通道三平台正式 release（ubuntu/windows/macos 全绿，非 draft）、镜像 ome/latest 拆 stable 触发（ohmycloud 跟进）、herdr 知会、`self update --stable` 验收（部署位升 0.2.0、doctor degraded 无 FAIL、catalog 同步、三态齐抽查 browser-harness） | 2026-09-10 |
| D29：claude linux pattern 修复（#10 缺口 1） | 已完成 | 根因：resolve 恒按 asset_pattern 对 release 清单重筛（pin 只锁 tag），pattern 写 x86_64 而官方资产名 claude-linux-x64.tar.gz；两行 pattern 修正，独立 TOML 解析核验三平台各唯一命中、SHASUMS256 与 pin sha 逐字一致、镜像 tar 单文件 claude 居根 | 2026-09-10 |
| D29：catalog_lint pattern 机检 | 已完成 | 新规则「pin 资产名必被同平台 asset_pattern 命中」（M014 同型拼写族零网络红灯）；真仓与夹具 2/2 绿，真仓无其他暗雷 | 2026-09-10 |
| D29：共识与补充裁落档 | 已完成 | R014 六（共识唯一权威：委托口径与实装态、三点确认、发版知会与桶段规范同步、集成优先级）；AGENTS 义务表发布行加知会 ohmycloud、边界加委托半句；ROADMAP 集成优先级节与阶段六；PRD/GOAL/CHANGELOG/INDEX 同步 | 2026-09-10 |
| D29：回执与坑合并 | 已完成 | 回执经 herdr 会话同步（三点确认与修复通报，通道更替裁当日生效）；M014 增补同型、R001 checklist 一补 pattern 逐字核对 | 2026-09-10 |
| D28：toolver 探测面迁 catalog | 已完成 | catalog.rs 加 probe_args/probe_pattern 字段；toolver.rs 删两 match 表改读 Tool；12 处调用方签名跟进（docker/vsbuild relaunch_elevated 与 ensure_machine_path 加 def 参数） | 2026-09-09 |
| D28：46 节回填与 fixtures 同步 | 已完成 | .tools/inject-probe-d28.py 幂等注入 46 节（8 特例 probe_args）；fixtures 三节补；catalog 头注释 41 改 46 顺修 | 2026-09-09 |
| D28：catalog 结构机检测试 | 已完成 | tests/catalog_lint.rs 两测绿（真仓加夹具）；toolver 单测改构造 Tool 期望值不动；install 沙盒 jq 节补 probe_pattern（幂等短路依赖探测） | 2026-09-09 |
| D28：文档同步与收口 | 已完成 | R001 字段两行加「入册 checklist」节（五，evergreen 顺移六，INDEX 交叉引用同步）；CHANGELOG/INDEX/.tools README；cargo test 全绿（91 加 28 加 2 加 2 加 2 加 7 加 7）加 clippy 加四件套；真机新构建 status 探测抽查八工具全对 | 2026-09-09 |
| D27：种子上传自维护（路线 A 加 B） | 已完成 | `.tools/seed.py` uv 单件（--plan 本机真测 98 对象 93 synced，抓出 gitleaks 缺种与 lightpanda mac 错配）；build.yml mirror job 加 seed-mirror.yml；R014 五节同步；首次 push CI 到货即镜像方验收关账 | 2026-09-08 |
| D26：gitleaks 入册 | 已完成 | security 类第三员（age/sops 管保管、gitleaks 管泄漏检测）；checksums.txt 锚、三平台单二进制、toolver 正则；win 真机绿 sha 三方一致；计数 45 改 46；种子三键随派镜像 | 2026-09-08 |
| D25：ome skill 逐工具自适应引导 | 已完成 | 用户两裁（扩展 ome skill 面、全 45 工具起稿）加自适应补裁（路径/env/数据目录实测）；catalog 四 guide 字段加渲染器重构；顺修落盘被静态版覆盖 bug；本机渲染实证与门禁全绿 | 2026-09-08 |
| D24：lightpanda 入册并本机先装 | 已完成 | 用户裁「先本机安装」：WSL 落 ~/.local/bin（sha 与 GitHub digest 逐字一致，version 实测 0.4.0）；catalog linux/mac pin 0.4.0（win 空态、无 sums 用 digest 锚、latest 被 nightly 占据须显式 --tag）；toolver version 子命令正则；计数 44 改 45；种子差集随派镜像 | 2026-09-08 |
| D23：browser-harness 与 reader 重入册 | 已完成 | 撤 09-07 暂不接管裁；reader 三平台 v0.6.0（.sha256 边车锚）win 幂等绿；browser-harness 走新 npm-tgz 通道（tgz 过锚下载加 npm install -g，bin bh，探测 PATH 现查加 cmd shim cmd /c）；win 真机双绿、status 三态齐、计数 42 改 44；种子差集随派镜像 | 2026-09-08 |
| 五端验收配合（用户裁：ome 最新版安装加更新，ohmycloud 执行） | 已完成 | 矩阵定稿执行**全过**：A 面本机 omc tool install（首次被锚校验正确拦下 CF 陈旧缓存，镜像侧已修 query 击穿）、wsl 裸装、lan 三端锚逐字吻合；B 面 OME_MIRROR=1 五端过（本机 updated 至 f3c41009，余 current，dev 段红线未破）；终态锚三平台全等；ome 侧跟进落地：镜像段请求统一缓存击穿 query | 2026-09-08 |
| herdr 0.9.0 升级收尾与种子派单 | 已完成 | 服务器经 schtask 会话外厂商更新器落 0.9.0、用户重启切 0.9.0（protocol 22，会话存活）；厂商安装器顺带升 ome 装位，update 幂等确认；PATH 收敛撤厂商目录优先级回 ome 单轨；`herdr integration install claude` 刷 hook 与 settings 种子；status 三态全绿；镜像 win 双档在位、linux/mac pin 仍 0.8.2 随对应机器升级 | 2026-09-08 |
| ohmyenv-rs#6：四家 install 幂等实证 | 已完成 | 部署 ome 含 D07（BD367811）；四家 `ome install` 全 skipped exit 0（claude 2.1.251 / codex 0.151.0 / grok 1.0.13 / kimi 0.39.1），存量纳管语义实证，关单 | 2026-09-08 |
| ohmyenv-rs#9：rclone 入册（D22） | 已完成 | 三平台节加 SHA256SUMS 锚加 zip/zip-bin 布局；toolver 补探测正则；win 1.75.1 真机安装绿（sha=官方清单逐字一致）、linux/mac pin 按官方清单回填；计数 41 改 42；关单回执与种子三键随派 | 2026-09-08 |
| D21：ohmycloud 三面对齐 ISSUE | 已完成 | ohmycloud#8：分发/更新/安装 ome 自身与在管软件；姊妹 #6 | 2026-09-07 |
| D20：ISSUE 派种子清单 | 已完成 | R014；ohmycloud#5（85 对象差集）；本仓跟踪曾为 #8 | 2026-09-07 |
| D20：ohmycloud 通报后镜像闸门 | 已完成 | 通报三处出入已域面复核；`OME_TEST_MIRROR=1` 两测绿（zoxide win、rmux mac 回落 sha=pin）；ffmpeg 边车与 pin 一致 | 2026-09-07 |
| 镜像种子前置：sha 补 pin | 已完成 | zoxide/ffmpeg linux_sha256 官方资产哈希回填（与 GitHub digest 一致）；HEAD 404 后派 ohmycloud#7 | 2026-09-07 |
| D20：ohmycloud#7 补种通报后闸门 | 已完成 | 闸门条件 2026-09-08 镜像回执确认（v2.4 台账 93/93，zoxide linux 与 ffmpeg win/linux 三资产 HEAD 200 实证）；随 D08 第二批一并落地（见下行） | 2026-09-08 |
| D08 第二批：latest 段边车锚回落 | 已完成 | `download_latest_with_sidecar` 落码（rust / vsbuild；先边车后资产，边车取不到拒绝无校验下载）；selfupdate 边车权威统一至 download 层；OME_TEST_MIRROR 6/6 绿（zoxide linux 回归、ffmpeg HEAD 在位）；ohmyenv-rs#7 回执 | 2026-09-08 |
| D08 第二批收尾：ome dev 通道断源兜底 | 已完成 | ohmycloud#9 补种 ome/dev 段（三资产加三边车，两段同源实证）；补 self update 断源锚链用例（锚取 ome/dev 边车，OME_TEST_MIRROR 7/7 绿）并关 ohmyenv-rs#7 | 2026-09-08 |
| D19：ohmycloud 分发兄弟仓 | 已完成 | 活文档定位：ohmycloud 是资源分发基建，ome 消费 env.ohmygh.com | 2026-09-07 |
| D18：活文档定位 | 已完成 | 不再有 ohmypwsh 现役依赖；D05 M6 与切片 4 取消 | 2026-09-07 |
| D18：代码与真机测试脱离 | 已完成 | heal 路由文案、R004 oracle、tests/real.rs 对照本机部署态；import-catalog 退役 | 2026-09-07 |
| D18：本机命令面验收 | 已完成 | doctor verdict=degraded exit 0；status/query/pin/verify 9 PASS；heal --dry-run；install ffmpeg skipped；skill；--llms | 2026-09-07 |
| D17：去掉 package | 已完成 | 删除 CLI 与 src/package.rs；文档同步 | 2026-09-07 |
| D16：去掉 daily 命令与编排 | 已完成 | CLI / status.rs 日常更新 / 文档与 --llms | 2026-09-07 |
| D15：去掉 deploy，install 全包 | 已完成 | 下载 + PATH + 注册表 + 配置；update 为 install 到最新 | 2026-09-07 |
| D15：CTA 与现役文档 | 已完成 | --llms / SKILL / AGENTS / R013 回到 install | 2026-09-07 |
| D15：测试改命令 | 已完成 | cli 25 绿；linux 闭环改为 install 含 PATH | 2026-09-07 |
| D14：ffmpeg catalog 入册 | 已完成 | Windows GyanD essentials 9.0.1；Linux BtbN n9.0 GPL；mac ARM 空态 | 2026-09-07 |
| D14：toolver 与文档计数 | 已完成 | 正则加样例；40 改 41、cli 17 改 18 | 2026-09-07 |
| D14：Windows 真机 install | 已完成 | query 9.0.1、install 幂等 skipped、status path=true；bin 含 ffmpeg/ffprobe/ffplay | 2026-09-07 |
| mac 分发测试（D36 验收） | 已完成 | lan-mac 断源双测绿（darwin 用例新增）、升级与实跑验收、install 幂等；回执 ohmycloud#3 | 2026-09-07 |
| D11：doctor 配置健康五项 | 已完成 | bunfig/goproxy/cargo-mirror/rust-relocate/telemetry（判据与 heal 同源、密钥不管、只管安装部署与默认配置）；真机 goproxy 缺口抓到 | 2026-09-07 |
| D09-1：根 SKILL.md | 已完成 | 何时用 ome、命令图（三原语口径标注）、镜像兜底与幂等语义；INDEX 登记 | 2026-09-07 |
| D09-2：ome --llms | 已完成 | 全局 flag 打印紧凑命令清单（与 SKILL.md 命令图同源同步）；子命令改可选（缺子命令给 agent 友好错误） | 2026-09-07 |
| D09-3：CTA 下一步建议 | 已完成 | doctor 缺口（missing 建议 install）与 status 漂移（建议 update，agent 类排除）走 stderr HINT，不进冻结数据面；真机 HINT 可见 | 2026-09-07 |
| 切片 1：catalog agent 条目建模 | 已完成 | 四家入册（pin/sha 迁 oma 实证值；双渠道的 CDN 兜底与 kimi manifest 渠道留切片 3）；taxonomy 九类（用户裁定 uv 是 python 运行时管理兼运行时、fnm 纯 node 管理）；install agent 存量纳管跳过与 status PATH 探测同步落地；R001/INDEX/AGENTS/README 同步 | 2026-09-05 |
| 切片 2：doctor 三层重构 | 已完成 | 系统层（os/arch/avx 三指令集，oma caps 同口径）、agent 层（binary/version/locked/drift/token 五字段；token 只探凭据文件不取设置不读环境变量，grok/kimi 实证判据、claude/codex 无稳定判据如实 na）、依赖层（九类分组 tools/missing/drift）加现有十项归 check 节；三态采集一次复用；85 加 26 测试全绿 | 2026-09-05 |
| 切片 3：install/deploy 幂等覆盖 agent 与分发兜底链（D08 并入） | 已完成 | 渠道：download_asset_with_mirror（官方失败回落镜像、有锚才回落）加双端断源闸门绿；布局：四家 force 沙盒真装全中（kimi 根级 kimi.exe 悬项实证、codex 嵌套 bin）；语义：update/daily 对 PATH 在位 agent 跳过（纳管同口径），verify 无 agent 专属维度（部署域维度注册表不涉逐工具）；R004 登记 OME_TEST_MIRROR；WSL status 自愈复验绿（zoxide/sheldon installed 回写） | 2026-09-07 |
| 审查缺陷收口 | 已完成 | pin/sha 同 tag、下载 .part、CDN tag、selfupdate 通道、doctor 判据、localbin16、Unix PATH、mac_exe、文档四原语 | 2026-09-07 |
| 切片 4：oma 迁册配合与跨仓收口 | 已取消 | D18：不再跨仓收口；oma deprecated 不在本仓范围 | 2026-09-07 |
| dotfiles 吸收批（S004） | 已完成 | zoxide/sheldon 入册（43 工具）；zoxide win 侧闭环、WSL 二进制就位（status 回写待新 dev 资产自愈）；下载链模式进 D08；S004 落档、INDEX/AGENTS/README 同步 | 2026-09-07 |
| D07 立项登记 | 已完成 | 追问链三轮六裁；PRD D07、GOAL 起点锚点切换（完整迁移 M6 挂队列）、PLAN 四切片 | 2026-09-05 |
| M6 ohmypwsh 链退役配合 | 已取消 | D18：源项目不存在，退役配合无对象 | 2026-09-07 |
| 文档体系 evo 对齐重建（D06） | 已完成 | PRD 全量追溯（D01..D06）、proven 归档节、ROADMAP、AGENTS 义务表与瘦身、INDEX 磁盘对账重整、diary 禁字清剿；evo check 13 项全绿、四件套全绿 | 2026-09-05 |

### 已完成批次

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 写作规范转换 | 已完成 | 《中英文 Markdown 技术文档写作规范》v1.0 全文转项目规范（用户裁决）：G001 v2 四类禁字符硬禁令与豁免区、mdcharlint.py 进四件套门禁（默认豁免 diary/proven）、非归档区 105 处存量清剿、四门禁全绿；跨仓规范与集成 ISSUE 矩阵（ohmypwsh#7、ohmyagents#2#3、ohmycloud#1） | 2026-09-02 |
| 七类 taxonomy 定稿 | 已完成 | 37 工具归七类（操作编排/运行时/编译器/运行时衍生/多路复用/远程服务/命令工具依赖，用户四轮裁决定稿）：catalog 节序即类序、ome 自管条目（extract=ome-self，clap version）、status 七组头、fixture 与黄金五件重生成（linux WSL 真机核对）；oma/omcf 待兄弟仓集成、reasonix 裁决不入册 | 2026-09-02 |
| openssh 接管与 doctor 修正 | 已完成 | Windows OpenSSH MSI 型条目（本机 10.0p2 无缝幂等接管、ssh -V stderr 探测、resolve pin 版本权威加固）；doctor 两类误报修正（uv-git sha 豁免、派生资产白名单）与本机清理（PATH 死链 gopath/bin、真孤儿 492MB），doctor 终态 9 项全 OK | 2026-09-02 |
| issue 驱动批与五端闭环 | 已完成 | bun mac 字段族（lan-mac verify 7/7，#2 关闭）；query sha256 契约冻结与 package all 容错（#4）；CI 真网限流修复（resolve GH_TOKEN 注入与测试步环境）后五端 self update 闭环 | 2026-09-02 |
| rust 接管建模 | 已完成 | rustup 引导器型条目（rsproxy 直链 evergreen 无 pin、stable 滚动即更新）；rustup.rs 迁移 set-rust.ps1 全语义（四用户环境变量重定位 EnvRoot、init 引导、update stable 保最新、cargo sparse 镜像 config.toml、cargo bin 用户 PATH）；heal dev-rust 键转原生 install；query/pin/update/daily/doctor evergreen 分支齐接；真机幂等验收 | 2026-09-02 |
| M4 heal 移植 | 已完成 | heal-map.psd1 42 键迁嵌入注册表（44 行，toolRoot/aria2 平台分列）四类归宿：install 类原生安装（pin 驱动幂等，rust 建模后 16 行）、dsKey/akKey/bunfig/goproxy 原生移植（heal-keys/heal-mirror 逐字节语义）、agent 域 12 键休眠（裁决）、非 ome 域 8 行路由提示（secret-guard/残留/compileMatrix/POSIX aria2）；mac-* 4 键别名归一；`ome heal [dim|all] [--dry-run]`；验收双平台闭环（Windows aria2/bunfig/dsKey + WSL go/dsKey 破坏-自愈-verify PASS、heal all 两连零 diff） | 2026-09-02 |
| M3 第二批：verify-five-ends 接线 | 已完成 | ohmypwsh 三端部署域维度行改透传 ome verify（ome=FAIL 兜底、compileMatrix 引用 ome 结果）；复验四端 73 项全绿，ps1 收敛为编排加密钥域 | 2026-09-01 |
| lan-win 端 ome 下发 | 已完成 | 固化 sync-ome-lanwin.ps1（sha 对比按需传二进制加 catalog 每次同步）；远端 verify 9 项全 PASS（顺带修 gh 漂移），M2 四端齐 | 2026-09-01 |
| M3 第一批：ome verify | 已完成 | 部署域维度注册表（Windows 9 维、POSIX 7 维），catalog 三态加文件存在判定，`dim=PASS/FAIL` 收割行与 ps1 同构；三端全 PASS 与 ps1 双跑对账一致；「WSL 探测缺口」经平台严格化自解 | 2026-09-01 |
| M2 WSL 与 lan-linux 批次 | 已完成 | WSL 原生构建自部署加二进制下发（Sync sha 对比按需传），install all 实证 WSL 31 项、lan-linux 24 项全幂等；lan-win 收尾另列待办；M5 远端通道以「ohmypwsh 编排调系统 ssh」形态达成（agent 归属已按裁决移出） | 2026-09-01 |
| go/zig Windows 安装建模 | 已完成 | cdn 直链（go.dev/dl、ziglang.org/download）加 zip-dir 提取型（zig 版本目录不展平）；pin 四键含 sha 回填，install 幂等跳过；status 28/31 完美接管 | 2026-09-01 |
| 遥测关闭 | 已完成 | pwsh：msi 属性 DISABLE_TELEMETRY 之外补 POWERSHELL_TELEMETRY_OPTOUT=1 与 POWERSHELL_UPDATECHECK=Off；dotnet：DOTNET_CLI_TELEMETRY_OPTOUT=1；新能力 set_user_env_var（HKCU/profile 标记块）挂 install 双路径钩子，注册表三值实证 | 2026-09-01 |
| Windows 回接与基线门禁 | 已完成 | mac 10 笔拉回后全门禁绿（76 测试、fmt/clippy、md 三件套、校验器「托管 26 节与 psd1 一致」）；修 Windows 真仓回归：仅 linux 字段工具（shellcheck）无通用 exe 致 status 全挂，加 platform_managed 容忍（status 空态行、install/update/daily/pin/query 跳过、package 拒绝），M106 记 M003 | 2026-09-01 |
| M1 后续：mac 完美收敛 | 已完成 | 目录型运行时布局实证回填（pwsh 装入 powershell/7、rmux 装入 ~/.local 树、zig 装入 ~/zig 版本目录、go 装入 ~/.local/go，新增 zip-bin/targz-dir/tarxz-dir 提取型）；extra_bins 多二进制（age-keygen/sg/uvx 实证）；cdn pattern 平台族修 vault；go/zig version 子命令与无前缀 tag 修复；sops/uv/vault/go mac pin 补齐（starship 真装、vault/uv 升级接管）；终态 18 工具 locked=installed、pwsh 版本对齐、win-only 与 official 布局如实空态 | 2026-09-01 |
| M0 数据主权与回流 | 已完成 | psd1 Pos 侧一次性回流（19 在管工具 linux 静态族+pin、16 工具 mac pin，钉版本 pattern 解开为 [0-9.]+）；pin runtime 平台分列（访问器/回写/sha 基准/status/update 全链切平台键，黄金双 oracle）；import-catalog.ps1 改只校验不再生（冲突报错、ome 增补放行、平台族完整性）；go/zig/shellcheck 补录（kimi 按不管理 agent 裁决剔除，共 31 工具）；同步纪律定案入 R001 | 2026-09-01 |
| M1 mac 字段族与真机验证 | 已完成 | Tool 增 `mac_*` 族、`effective_*` 三分支回退链（mac、linux、通用）、exe 双语义修复；jq/fnm 首批 mac 补录；R011 六项真机验证全绿（70 测试、install/deploy 幂等接管在位 jq、self-deploy 部署态解析、package fnm、gnu 交叉 check）；真机门禁修三坑（exe 双拼、clippy cfg、测试沙盒漏 catalog 记 M102） | 2026-09-01 |
| vsbuild 接管 | 已完成 | evergreen 引导器（无 pin 无 sha）+ gsudo 自动提权 + 机器级 PATH + MSBuild 稳定探测；74 测试全绿、真机幂等空转验证；Windows SDK 仍留 ohmypwsh（ISO 分离） | 2026-09-01 |
| 新址 clone 与基线门禁 | 已完成 | D:\ohmyenv-rs 基线 993e77b；修 CRLF 检出敏感测试（夹具 CRLF 检出时 `\r\n` 双转）后门禁全绿 | 2026-09-01 |
| 安装形态整改 | 已完成 | 自部署进 `%LOCALAPPDATA%\Programs\ome`、catalog 同步 `%LOCALAPPDATA%\ohmyenv`、旧 PATH 残留清理；catalog 解析扩四级；68 测试全绿 | 2026-09-01 |
| 承接完整迁移登记 | 已完成 | R012 降级标注（被 P0026 取代）、三原语切换、INDEX 与 diary 登记 | 2026-09-01 |
| ohmypwsh catalog 与部署脚本对齐 | superseded | 被完整迁移裁决取代（2026-09-01）：数据改走 M0 单向回流，不再双向对齐 | 2026-09-01 |
