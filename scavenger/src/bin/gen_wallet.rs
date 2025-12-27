use solana_sdk::signature::{Keypair, Signer};
use std::error::Error;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    // 生成新钱包
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    
    // 转换为 JSON 格式 (bytes 数组)
    let bytes = keypair.to_bytes();
    let json_string = serde_json::to_string(&bytes.to_vec())?;
    
    // 保存文件名
    let filename = "withdrawal_wallet.json";
    let mut file = File::create(filename)?;
    file.write_all(json_string.as_bytes())?;
    
    println!("✅ 钱包生成成功!");
    println!("📂 私钥文件: ./{}", filename);
    println!("Tb 公钥 (Address): {}", pubkey);
    println!("⚠️  请务必备份该文件，丢失无法找回!");
    
    Ok(())
}
