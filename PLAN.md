# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D34 云端清单防 MITM 的非对称校验（用户 2026-09-10「云端这个 catalog 文件应该要想办法
> 校验防 MITM」「本地和云端一起非对称校验，像 SOPS 的方式」，用户令「开工」取建议默认）。

### 依据

- 研究在档（S006）：sha 边车只证传输与缓存，同源者可同时替换清单与边车；要证来源必须非对称签名。SOPS 的真身是加密加文件内 MAC（不是签名），可吸收的是密钥分工；apt 与 Helm 与 HashiCorp 用分离签名加预置信任根，Sparkle 用公钥内嵌二进制，TUF 另加过期与版本防回滚。
- 取建议默认（用户令「开工」）：密钥载体新建 Ed25519 签名密钥对（minisign 格式）；校验强度取「有签名但验不过即阻断，缺签名件只告警」（本地 pin 回写会撤签名，避免误伤正常回写）。
- 边界：其他机器不装私钥，公钥随 ome 二进制分发（`ome init` 与 `self update` 自动到位）；回滚防护（清单内单调序号）留作下一步。

### 方案骨架

1. **密钥与工具**：`.tools/catalog-sign`（Rust 子工程，minisign 库）提供 keygen 与 pubkey 与 sign 与 verify；keygen 生成本机私钥 `~/.config/ome/catalog-signing.key`（不进仓库）与公钥 `.tools/catalog-sign/catalog-signing.pub`（进仓库供审计与 CI 自检）；公钥 base64 内嵌 `src/catalog.rs`，单测机检「仓库公钥文件与内嵌公钥一致」，防两处漂移。
2. **签发流水**：seed-mirror 加签名步（rust-toolchain 加 `cargo run --manifest-path .tools/catalog-sign`），私钥取 GitHub Secret `CATALOG_SIGNING_KEY`，缺密钥即失败不发布未签名清单；`seed.py` 把 `.minisig` 随清单与边车一并入镜，缺签名件打印告警。
3. **客户端校验**：`minisign-verify` 依赖加 `verify_with_embedded_keys`（支持多公钥，便于轮换）；拉取落位前过 sha、解析、验签三重，任一不过拒收；签名件随刷新落位到 `<清单>.minisig`；每次命令加载前巡检验签（catalog 子命令豁免，保证 status 可见与 sync 可自愈）；`write_pin` 与 init 的 catalog 同步会撤掉签名件，避免留「签过但内容已变」的假凭证。
4. **命令面**：`catalog status` 增 `signature`（valid / invalid / missing）与 `pubkey`（内嵌公钥 key id）字段；`catalog sync` 变为「验签通过才落位」。
5. **文档同步**：R001（sha 与签名的分工、云端可见性加签名件）与 R013（字段与退出码面）；README 与 SKILL 与 AGENTS 补签名语义；CHANGELOG、PRD/GOAL/TODO、diary；S006 保持研究原档。

### 完成定义

- 云端清单未签名或签名不符一律拒收（含自举与自动刷新路径），本地运行态副本签名不符即阻断命令。
- 其他机器零手工：公钥随二进制到位，`ome catalog status` 应报 `signature=valid`。
- 密钥轮换路径在档（双公钥过渡），私钥泄露处置（换钥发版加重签云端件）在档。
- 单测覆盖验签正反例与公钥一致性机检，gated 真网测覆盖签名件落位与篡改拒收。

### 验收

- `cargo test` 全绿（含新增单测与 gated 真网测）、`cargo clippy` 干净、`rumdl check .` 与 `.tools` 三扫描绿。
- 推 main 后 seed-mirror 首发签名件；本机 `ome catalog sync` 验签通过落位、`catalog status` 报 `signature=valid`。
- 篡改实证：改动运行态副本后普通命令被阻断（签名不符），`ome catalog sync` 能取回云端件自愈。
- 黄金文件与夹具不破（夹具 catalog 不带签名件，签名巡检只对运行态副本告警）。
