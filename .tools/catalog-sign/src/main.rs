//! catalog-sign：ome 云端清单（catalog）的 minisign 格式签名工具（D34）。
//!
//! 用法（本地与 CI 同一工具，避免第三方 CLI 版本漂移）：
//!   cargo run --manifest-path .tools/catalog-sign/Cargo.toml -- keygen -p <pub> -s <sec>
//!   cargo run --manifest-path .tools/catalog-sign/Cargo.toml -- pubkey -s <sec>
//!   cargo run --manifest-path .tools/catalog-sign/Cargo.toml -- sign -s <sec> -m <file> [-x <sig>]
//!   cargo run --manifest-path .tools/catalog-sign/Cargo.toml -- verify -p <pub> -m <file> [-x <sig>]
//!
//! 默认签名文件为 `<file>.minisig`（minisign 惯例）。私钥只在本机（`~/.config/ome/`）与
//! CI 密钥库（GitHub Secret）出现；公钥进仓库与 ome 二进制内嵌，其他机器只需公钥即可校验。
//!
//! 退出码：0 成功；1 失败；2 用法错误。

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::exit;

use minisign::{KeyPair, PublicKeyBox, SecretKeyBox, SignatureBox};

fn usage() -> ! {
    eprintln!(
        "用法:\n  \
         catalog-sign keygen -p <pub> -s <sec>\n  \
         catalog-sign pubkey -s <sec>\n  \
         catalog-sign sign -s <sec> -m <file> [-x <sig>]\n  \
         catalog-sign verify -p <pub> -m <file> [-x <sig>]"
    );
    exit(2)
}

/// 取紧随选项之后的参数值。
fn value_of(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn sig_path_of(args: &[String], msg: &PathBuf) -> PathBuf {
    value_of(args, "-x").map(PathBuf::from).unwrap_or_else(|| {
        let mut p = msg.clone().into_os_string();
        p.push(".minisig");
        PathBuf::from(p)
    })
}

fn read_secret(path: &PathBuf) -> Result<minisign::SecretKey, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("读私钥失败: {}: {e}", path.display()))?;
    SecretKeyBox::from_string(&text)
        .map_err(|e| format!("私钥格式不合法: {e}"))?
        .into_unencrypted_secret_key()
        .map_err(|e| format!("私钥解密失败（本工具只支持无口令私钥）: {e}"))
}

fn read_public(path: &PathBuf) -> Result<minisign::PublicKey, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("读公钥失败: {}: {e}", path.display()))?;
    PublicKeyBox::from_string(&text)
        .map_err(|e| format!("公钥格式不合法: {e}"))?
        .into_public_key()
        .map_err(|e| format!("公钥解析失败: {e}"))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(String::as_str) else {
        usage()
    };
    match cmd {
        "keygen" => {
            let (Some(pk), Some(sk)) = (value_of(&args, "-p"), value_of(&args, "-s")) else {
                usage()
            };
            let pk_file = File::create(&pk).unwrap_or_else(|e| {
                eprintln!("建公钥文件失败: {pk}: {e}");
                exit(1)
            });
            let sk_file = File::create(&sk).unwrap_or_else(|e| {
                eprintln!("建私钥文件失败: {sk}: {e}");
                exit(1)
            });
            // 写文本 box（minisign 惯例：untrusted comment 行加 base64 行），不用 to_bytes（原始二进制）
            match KeyPair::generate_unencrypted_keypair()
                .and_then(|kp| Ok((kp.pk.to_box()?.to_string(), kp.sk.to_box(None)?.to_string())))
                .and_then(|(pk_text, sk_text)| {
                    use std::io::Write;
                    (&pk_file).write_all(pk_text.as_bytes())?;
                    (&sk_file).write_all(sk_text.as_bytes())?;
                    Ok(())
                }) {
                Ok(()) => println!("pub={pk}\nsec={sk}"),
                Err(e) => {
                    eprintln!("生成密钥对失败: {e}");
                    exit(1)
                }
            }
        }
        "pubkey" => {
            let Some(sk) = value_of(&args, "-s") else { usage() };
            let sk = read_secret(&PathBuf::from(&sk)).unwrap_or_else(|e| {
                eprintln!("{e}");
                exit(1)
            });
            match minisign::PublicKey::from_secret_key(&sk)
                .and_then(|pk| pk.to_box())
                .map(|b| b.to_string())
            {
                Ok(text) => print!("{text}"),
                Err(e) => {
                    eprintln!("导出公钥失败: {e}");
                    exit(1)
                }
            }
        }
        "sign" => {
            let (Some(sk), Some(msg)) = (value_of(&args, "-s"), value_of(&args, "-m")) else {
                usage()
            };
            let msg_path = PathBuf::from(&msg);
            let sk = read_secret(&PathBuf::from(&sk)).unwrap_or_else(|e| {
                eprintln!("{e}");
                exit(1)
            });
            let reader = BufReader::new(File::open(&msg_path).unwrap_or_else(|e| {
                eprintln!("读清单失败: {msg}: {e}");
                exit(1)
            }));
            let comment = format!("ome catalog signature: {}", msg_path.display());
            let sig = minisign::sign(None, &sk, reader, Some(&comment), Some(&comment))
                .unwrap_or_else(|e| {
                    eprintln!("签名失败: {e}");
                    exit(1)
                });
            let out = sig_path_of(&args, &msg_path);
            match std::fs::write(&out, sig.into_string()) {
                Ok(()) => println!("sig={}", out.display()),
                Err(e) => {
                    eprintln!("写签名失败: {}: {e}", out.display());
                    exit(1)
                }
            }
        }
        "verify" => {
            let (Some(pk), Some(msg)) = (value_of(&args, "-p"), value_of(&args, "-m")) else {
                usage()
            };
            let msg_path = PathBuf::from(&msg);
            let sig_path = sig_path_of(&args, &msg_path);
            let pk = read_public(&PathBuf::from(&pk)).unwrap_or_else(|e| {
                eprintln!("{e}");
                exit(1)
            });
            let sig_text = std::fs::read_to_string(&sig_path).unwrap_or_else(|e| {
                eprintln!("读签名失败: {}: {e}", sig_path.display());
                exit(1)
            });
            let sig = SignatureBox::from_string(&sig_text).unwrap_or_else(|e| {
                eprintln!("签名格式不合法: {e}");
                exit(1)
            });
            let reader = BufReader::new(File::open(&msg_path).unwrap_or_else(|e| {
                eprintln!("读清单失败: {msg}: {e}");
                exit(1)
            }));
            match minisign::verify(&pk, &sig, reader, true, false, false) {
                Ok(()) => println!("verify=ok"),
                Err(e) => {
                    eprintln!("验签失败: {e}");
                    exit(1)
                }
            }
        }
        _ => usage(),
    }
}
