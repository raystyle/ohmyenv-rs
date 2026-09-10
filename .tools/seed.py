# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""seed.py - catalog 软件集与 ome 自产产物的 env.ohmygh.com（R2）种子同步（D27/D41 路线 A 加 B）

口径（R014 延续）：
- 唯一权威 catalog\tools.toml：version + asset + sha256 三键齐才入镜；无 sha 进 pending 队列；
  hold 与 evergreen（ome-self/vsbuild/rustup）不入镜。
- 边车自算即锚：`<hex 小写>空两格<asset>` 同名 .sha256 边车，上传前资产先过 catalog sha 锚校验。
- diff 走公网域面（GET 边车带 ?t= 时间戳击穿 + HEAD 资产），不需要 R2 凭据；上传走 rclone（CI 内）。
- rclone 配方：provider=Cloudflare、endpoint=R2_ENDPOINT、NO_CHECK_BUCKET 必带（受限 token 无建桶权）、
  Cache-Control: public, max-age=60（消费侧 ?v=/?t= 击穿双保险）。

用法（uv 零安装，runner 预装 uv）：
  uv run --script .tools/seed.py --plan            # 全 catalog 域面 diff（只读，无凭据可跑）
  uv run --script .tools/seed.py                   # diff 加上传（需 R2_* 环境变量与 rclone）
  uv run --script .tools/seed.py --ome-dev --tag dev         # 路线 A：dev 产物灌 ome/dev 沙滚段
  uv run --script .tools/seed.py --ome-stable --tag v0.2.0   # 路线 A：v* 正式产物灌 ome/stable 段（oma 同型，latest 段退役）

环境变量：R2_ACCESS_KEY_ID / R2_SECRET_ACCESS_KEY / R2_ENDPOINT / R2_BUCKET（上传必需）；
GH_TOKEN 可选（公开仓不需要）。

退出码：0 全同步或 plan；1 有失败项；2 catalog 解析失败。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "catalog" / "tools.toml"
DOMAIN = "https://env.ohmygh.com"
EVERGREEN_EXTRACT = {"ome-self", "vsbuild", "rustup"}
PLATFORMS = (("win", "", ""), ("linux", "linux_", "linux_"), ("mac", "mac_", "mac_"))
# 路线 A 的本仓三资产（CI 目标三元组；selfupdate 资产名同源）
OME_ASSETS = [
    "ome-x86_64-pc-windows-msvc.exe",
    "ome-x86_64-unknown-linux-gnu",
    "ome-aarch64-apple-darwin",
]
RCLONE_ENV = {
    "RCLONE_CONFIG_SEED_TYPE": "s3",
    "RCLONE_CONFIG_SEED_PROVIDER": "Cloudflare",
    "RCLONE_CONFIG_SEED_ENDPOINT": "R2_ENDPOINT",
    "RCLONE_CONFIG_SEED_ACCESS_KEY_ID": "R2_ACCESS_KEY_ID",
    "RCLONE_CONFIG_SEED_SECRET_ACCESS_KEY": "R2_SECRET_ACCESS_KEY",
    "RCLONE_CONFIG_SEED_NO_CHECK_BUCKET": "true",
}


def http_get(url: str, timeout: int = 30) -> tuple[int, bytes]:
    req = urllib.request.Request(url, headers={"User-Agent": "ome-seed", "Cache-Control": "no-cache"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read()
    except urllib.error.HTTPError as e:
        return e.code, b""
    except Exception as e:  # 网络抖动如实报
        return -1, str(e).encode()


def http_head(url: str, timeout: int = 30) -> int:
    req = urllib.request.Request(url, method="HEAD", headers={"User-Agent": "ome-seed"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status
    except urllib.error.HTTPError as e:
        return e.code
    except Exception:
        return -1


def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sidecar_text(sha_hex: str, asset: str) -> str:
    return f"{sha_hex.lower()}  {asset}\n"


def collect() -> tuple[list[dict], list[dict], list[str]]:
    """catalog -> （可入镜对象, pending_sha 队列, evergreen 排除名单）"""
    data = tomllib.loads(CATALOG.read_text(encoding="utf-8"))["tools"]
    objs: list[dict] = []
    pending: list[dict] = []
    evergreen: list[str] = []
    for name, t in data.items():
        ext = t.get("extract") or ""
        if ext in EVERGREEN_EXTRACT or t.get("hold"):
            evergreen.append(name)
            continue
        prefix_tag = t.get("tag_prefix") or ""
        for plat, pver, prel in PLATFORMS:
            repo = t.get(prel + "repo") or t.get("repo")
            version = t.get(pver + "version")
            asset = t.get(pver + "asset")
            sha = t.get(pver + "sha256")
            tag = t.get(pver + "tag") or (prefix_tag + version if version else None)
            if version and asset and sha:
                objs.append(
                    {"tool": name, "platform": plat, "repo": repo, "tag": tag,
                     "version": version, "asset": asset, "sha256": sha.upper()}
                )
            elif version or asset:
                pending.append({"tool": name, "platform": plat, "version": version, "asset": asset})
    return objs, pending, evergreen


def remote_state(tool: str, version: str, asset: str, sha: str) -> str:
    """域面状态：synced（边车值等且资产在）/ sidecar-missing / asset-missing / sha-drift / unreachable"""
    bust = str(int(time.time()))
    code, body = http_get(f"{DOMAIN}/{tool}/{version}/{asset}.sha256?t={bust}")
    if code == -1:
        return "unreachable"
    if code != 200:
        return "sidecar-missing"
    first = body.decode(errors="replace").split()
    if not first or first[0].lower() != sha.lower():
        return "sha-drift"
    if http_head(f"{DOMAIN}/{tool}/{version}/{asset}?v={sha[:16]}") != 200:
        return "asset-missing"
    return "synced"


def upload_pair(local_asset: Path, sha_hex: str, tool: str, version: str, dry: bool) -> bool:
    """资产加边车成对上传（rclone copyto；Cache-Control 双保险）"""
    header = ["--header-upload", "Cache-Control: public, max-age=60", "--s3-upload-cutoff", "64MiB"]
    ok = True
    for src, key in (
        (local_asset, f"{tool}/{version}/{local_asset.name}"),
        (None, f"{tool}/{version}/{local_asset.name}.sha256"),
    ):
        if src is None:
            side = local_asset.parent / f"{local_asset.name}.sha256"
            side.write_text(sidecar_text(sha_hex, local_asset.name), encoding="utf-8", newline="\n")
            src = side
        args = ["copyto", str(src), key, *header]
        env = dict(os.environ)
        for k, v in RCLONE_ENV.items():
            env[k] = os.environ[v] if v.startswith("R2_") else v
        if dry:
            print(f"[plan] rclone copyto {src.name} -> seed:$R2_BUCKET/{key}")
            continue
        bucket = os.environ["R2_BUCKET"]
        proc = subprocess.run(
            ["rclone", "copyto", str(src), f"seed:{bucket}/{key}", *header],
            env=env, capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(f"[FAIL] rclone copyto {key}: {proc.stderr.strip()[:300]}")
            ok = False
    return ok


def download_asset(repo: str, tag: str, asset: str, dest: Path) -> bool:
    url = f"https://github.com/{repo}/releases/download/{tag}/{asset}"
    code, body = http_get(url, timeout=600)
    if code != 200 or not body:
        print(f"[FAIL] 下载 {url}: HTTP {code}")
        return False
    dest.write_bytes(body)
    return True


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--plan", action="store_true", help="只 diff 不上传")
    ap.add_argument("--ome-dev", action="store_true", help="路线 A：dev 产物灌 ome/dev 沙滚段")
    ap.add_argument("--ome-stable", action="store_true", help="路线 A：v* 正式产物灌 ome/stable 段")
    ap.add_argument("--tag", default="dev", help="路线 A 的 release tag")
    args = ap.parse_args()
    dry = args.plan

    if args.ome_dev or args.ome_stable:
        repo = "raystyle/ohmyenv-rs"
        # 段域 oma 同型：段名与 self update 通道同名（ome/dev 沙滚、ome/stable 正式；
        # ome/latest 段退役，D30 封版拆分 2026-09-10 落地）
        segs = ("ome/dev",) if args.ome_dev else ("ome/stable",)
        results = {"synced": 0, "uploaded": 0, "failed": 0}
        with tempfile.TemporaryDirectory() as td:
            tdp = Path(td)
            for asset in OME_ASSETS:
                local = tdp / asset
                if not download_asset(repo, args.tag, asset, local):
                    results["failed"] += 1
                    continue
                sha = sha256_file(local)
                # 沙滚段无 version 目录：路径 <seg>/<asset>，无条件重灌（沙滚语义）
                ok = all(upload_pair_seg(local, sha, seg, dry) for seg in segs)
                results["uploaded" if ok else "failed"] += 1
        mode = "ome-dev" if args.ome_dev else "ome-stable"
        print(json.dumps({"mode": mode, **results}, ensure_ascii=False))
        return 0 if results["failed"] == 0 else 1

    objs, pending, evergreen = collect()
    if objs is None:
        return 2
    results = {"total": len(objs), "synced": 0, "uploaded": 0, "failed": 0}
    fails: list[str] = []
    with tempfile.TemporaryDirectory() as td:
        tdp = Path(td)
        for o in objs:
            state = remote_state(o["tool"], o["version"], o["asset"], o["sha256"])
            if state == "synced":
                results["synced"] += 1
                continue
            print(f"[{'plan' if dry else 'sync'}] {o['tool']}/{o['version']}/{o['asset']}: {state}")
            if dry:
                continue
            local = tdp / o["asset"]
            if not download_asset(o["repo"], o["tag"], o["asset"], local):
                results["failed"] += 1
                fails.append(o["asset"])
                continue
            actual = sha256_file(local)
            if actual.upper() != o["sha256"]:
                print(f"[FAIL] 锚校验不过 {o['asset']}: catalog {o['sha256'][:12]}… 实际 {actual[:12]}…")
                results["failed"] += 1
                fails.append(o["asset"])
                continue
            if upload_pair(local, actual, o["tool"], o["version"], False):
                results["uploaded"] += 1
            else:
                results["failed"] += 1
                fails.append(o["asset"])
    print(json.dumps({**results, "pending_sha": len(pending), "evergreen": len(evergreen),
                      "fails": fails}, ensure_ascii=False))
    return 1 if (not dry and results["failed"]) else 0


def upload_pair_seg(local_asset: Path, sha_hex: str, seg: str, dry: bool) -> bool:
    """沙滚段上传：<seg>/<asset> 与 .sha256 边车（无 version 目录）"""
    ok = True
    for name, content in (
        (local_asset.name, None),
        (local_asset.name + ".sha256", sidecar_text(sha_hex, local_asset.name).encode()),
    ):
        src = local_asset.parent / name
        if content is not None:
            src.write_bytes(content)
        key = f"{seg}/{name}"
        if dry:
            print(f"[plan] rclone copyto {src.name} -> seed:$R2_BUCKET/{key}")
            continue
        env = dict(os.environ)
        for k, v in RCLONE_ENV.items():
            env[k] = os.environ[v] if v.startswith("R2_") else v
        bucket = os.environ["R2_BUCKET"]
        proc = subprocess.run(
            ["rclone", "copyto", str(src), f"seed:{bucket}/{key}",
             "--header-upload", "Cache-Control: public, max-age=60"],
            env=env, capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(f"[FAIL] rclone copyto {key}: {proc.stderr.strip()[:300]}")
            ok = False
    return ok


if __name__ == "__main__":
    sys.exit(main())
