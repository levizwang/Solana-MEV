use bip39::{Mnemonic, Language};
use solana_sdk::signature::{Keypair, Signer, SeedDerivable};
use std::fs::File;
use std::io::Write;
use rand::RngCore;

fn main() {
    // 1. 生成 Entropy (16 bytes = 128 bits -> 12 words)
    let mut entropy = [0u8; 16];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut entropy);

    // 2. 生成 Mnemonic
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
    let phrase = mnemonic.to_string(); // bip39 2.0 uses to_string() or word_iter()
    
    println!("================================================================");
    println!("🔑 新钱包已生成 (New Wallet Generated)");
    println!("================================================================");
    println!("📝 助记词 (Mnemonic Phrase):");
    println!("{}", phrase);
    println!("================================================================");
    
    // 3. 生成 Seed (64 bytes)
    let seed = mnemonic.to_seed(""); 
    
    // 4. 生成 Keypair
    // 使用前 32 字节 (Ed25519 standard seed size)
    // 这与 solana-keygen new --no-bip39-passphrase 兼容
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&seed[0..32]);
    let keypair = Keypair::from_seed(&secret).unwrap();
    
    let pubkey = keypair.pubkey();
    let keypair_bytes = keypair.to_bytes(); // 64 bytes
    let json_string = serde_json::to_string(&keypair_bytes.to_vec()).unwrap();

    // 5. 保存文件
    // 覆盖之前的 withdrawal_wallet.json
    let filename = "../withdrawal_wallet.json";
    let mut file = File::create(filename).expect("Failed to create file");
    file.write_all(json_string.as_bytes()).expect("Failed to write file");

    println!("✅ 钱包私钥文件已更新: {}", filename);
    println!("Tb 公钥 (Address): {}", pubkey);
    println!("");
    println!("⚠️  重要说明:");
    println!("   由于原私钥是随机生成的，无法转为助记词。");
    println!("   我已经为您生成了一个【包含助记词的新钱包】并覆盖了原文件。");
    println!("   请务必保存好上面的助记词！");
    println!("================================================================");
}
