use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use tokio::fs;

use crate::{
    process_text_decrypt, process_text_encrypt, process_text_generate, process_text_sign,
    process_text_verify, CmdExecutor,
};

use super::{verify_file, verify_path};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum TextSubCommand {
    #[command(name = "sign", about = "Sign a message with a private key.")]
    Sign(TextSignOpts),

    #[command(name = "verify", about = "Verify a signed message with a public key.")]
    Verify(TextVerifyOpts),

    #[command(name = "generate", about = "Generate a random password.")]
    Generate(TextGenerateOpts),

    #[command(name = "encrypt", about = "Encrypt a message use cha-cha20-poly1305.")]
    Encrypt(TextEncryptOpts),

    #[command(name = "decrypt", about = "Decrypt a message use cha-cha20-poly1305.")]
    Decrypt(TextDecryptOpts),
}

#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,

    #[arg(long, default_value = "blake3", value_parser = parse_format)]
    pub format: TextFormat,
}

#[derive(Debug, Parser)]
pub struct TextVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,

    #[arg(short, long)]
    pub signature: String,

    #[arg(long, default_value = "blake3", value_parser = parse_format)]
    pub format: TextFormat,
}

#[derive(Debug, Parser)]
pub struct TextGenerateOpts {
    #[arg(short, long, default_value = "blake3", value_parser = parse_format)]
    pub format: TextFormat,

    #[arg(short, long, value_parser = verify_path)]
    pub output: PathBuf,
}

#[derive(Debug, Parser)]
pub struct TextEncryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,
}

#[derive(Debug, Parser)]
pub struct TextDecryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,
}

fn parse_format(format: &str) -> Result<TextFormat, anyhow::Error> {
    format.parse()
}

#[derive(Debug, Clone, Copy)]
pub enum TextFormat {
    Blake3,
    Ed25519,
    ChaCha20Poly1305,
}

impl FromStr for TextFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blake3" => Ok(TextFormat::Blake3),
            "ed25519" => Ok(TextFormat::Ed25519),
            "chacha20poly1305" => Ok(TextFormat::ChaCha20Poly1305),
            _ => Err(anyhow::anyhow!("Invalid text sign format")),
        }
    }
}

impl From<TextFormat> for &str {
    fn from(format: TextFormat) -> Self {
        match format {
            TextFormat::Blake3 => "blake3",
            TextFormat::Ed25519 => "ed25519",
            TextFormat::ChaCha20Poly1305 => "chacha20poly1305",
        }
    }
}

impl fmt::Display for TextFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signed = process_text_sign(&self.input, &self.key, self.format)?;
        println!("{}", signed);
        Ok(())
    }
}

impl CmdExecutor for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let verified = process_text_verify(&self.input, &self.key, &self.signature, self.format)?;
        if verified {
            println!("✓ Signature verified");
        } else {
            println!("⚠ Signature not verified");
        }
        Ok(())
    }
}

impl CmdExecutor for TextGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let keys = process_text_generate(self.format)?;
        match self.format {
            TextFormat::Blake3 => {
                let name = self.output.join("blake3.txt");
                fs::write(name, &keys[0]).await?;
            }
            TextFormat::Ed25519 => {
                let name = &self.output;
                fs::write(name.join("ed25519.sk"), &keys[0]).await?;
                fs::write(name.join("ed25519.pk"), &keys[1]).await?;
            }
            TextFormat::ChaCha20Poly1305 => {
                let name = self.output.join("chacha20poly1305.txt");
                fs::write(name, &keys[0]).await?;
            }
        }
        Ok(())
    }
}

impl CmdExecutor for TextEncryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encrypted = process_text_encrypt(&self.input, &self.key)?;

        println!("\n{}", encrypted);
        Ok(())
    }
}

impl CmdExecutor for TextDecryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decrypted = process_text_decrypt(&self.input, &self.key)?;
        println!("\n{}", decrypted);
        Ok(())
    }
}
