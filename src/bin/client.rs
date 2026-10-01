use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const SERVER_ADDR: &str = "127.0.0.1:6379";

const BANNER: &str = r#"
 ███╗   ██╗ ██████╗ ██████╗ ██╗
 ████╗  ██║██╔═══██╗██╔══██╗██║
 ██╔██╗ ██║██║   ██║██████╔╝██║
 ██║╚██╗██║██║   ██║██╔══██╗██║
 ██║ ╚████║╚██████╔╝██║  ██║██║
 ╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝╚═╝
 === Nori Database CLI v1.0 ===
"#;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\x1b[36m{}\x1b[0m", BANNER);
    println!(
        "Connecting to Nori server at \x1b[1m{}\x1b[0m...\n",
        SERVER_ADDR
    );

    let mut stream = match TcpStream::connect(SERVER_ADDR).await {
        Ok(s) => {
            println!("\x1b[32m[Connected Successfully!]\x1b[0m");
            println!(
                "Type \x1b[1;33mhelp\x1b[0m to see available local commands or start executing database commands.\n"
            );
            s
        }
        Err(e) => {
            eprintln!("\x1b[31m[Error]: Failed to connect to server: {}\x1b[0m", e);
            eprintln!("Make sure the Nori server is running on {}.", SERVER_ADDR);
            return Ok(());
        }
    };

    // Inisialisasi Rustyline Editor untuk REPL
    let mut rl = DefaultEditor::new()?;
    let history_path = "nori_history.txt";

    // Muat riwayat command jika ada
    if rl.load_history(history_path).is_err() {
        // Abaikan jika file history belum ada
    }

    loop {
        let readline = rl.readline("\x1b[1;35mnori>\x1b[0m ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                // 1. Tangani Local Commands (Tidak perlu dikirim ke server)
                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    println!("\x1b[33mDisconnecting... Goodbye!\x1b[0m");
                    break;
                }

                if trimmed.eq_ignore_ascii_case("clear") {
                    // Bersihkan layar terminal menggunakan ANSI escape code
                    print!("\x1b[2J\x1b[1;1H");
                    println!("\x1b[36m{}\x1b[0m", BANNER);
                    continue;
                }

                if trimmed.eq_ignore_ascii_case("help") {
                    println!("\n\x1b[1mAVAILABLE LOCAL COMMANDS:\x1b[0m");
                    println!("  \x1b[32mhelp\x1b[0m         - Menampilkan menu bantuan ini");
                    println!("  \x1b[32mclear\x1b[0m        - Membersihkan layar terminal");
                    println!("  \x1b[32mexit / quit\x1b[0m  - Keluar dari Nori CLI\n");

                    println!("\x1b[1mSERVER DATABASE COMMANDS:\x1b[0m");
                    println!(
                        "  \x1b[33mGET <key>\x1b[0m                     - Mengambil nilai berdasarkan key"
                    );
                    println!(
                        "  \x1b[33mSET <key> <val>\x1b[0m               - Menyimpan data (permanen)"
                    );
                    println!(
                        "  \x1b[33mSET <key> <val> EX <seconds>\x1b[0m  - Menyimpan data dengan TTL (Expired)"
                    );
                    println!(
                        "  \x1b[33mSETX <key> <val> <seconds>\x1b[0m  - Alternatif set dengan TTL"
                    );
                    println!(
                        "  \x1b[33mDEL <key>\x1b[0m                     - Menghapus key dari database"
                    );
                    println!(
                        "  \x1b[33mSTATS\x1b[0m                         - Melihat statistik cache hits/misses\n"
                    );
                    continue;
                }

                // Masukkan input yang valid ke history rustyline
                let _ = rl.add_history_entry(trimmed);

                // 2. Kirim perintah ke server via TCP
                if let Err(e) = stream.write_all(format!("{}\n", trimmed).as_bytes()).await {
                    eprintln!(
                        "\x1b[31m[Error]: Failed to send command to server: {}\x1b[0m",
                        e
                    );
                    break;
                }

                if let Err(e) = stream.flush().await {
                    eprintln!("\x1b[31m[Error]: Failed to flush stream: {}\x1b[0m", e);
                    break;
                }

                // 3. Baca response berformat tabel dari server
                let mut buffer = vec![0; 4096];
                match stream.read(&mut buffer).await {
                    Ok(n) if n == 0 => {
                        println!("\x1b[31m[Server disconnected]\x1b[0m");
                        break;
                    }
                    Ok(n) => {
                        let response = String::from_utf8_lossy(&buffer[..n]);
                        print!("\n{}", response);
                    }
                    Err(e) => {
                        eprintln!("\x1b[31m[Error]: Failed to read response: {}\x1b[0m", e);
                        break;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n\x1b[33mExiting (Ctrl-C)...\x1b[0m");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("\n\x1b[33mExiting (Ctrl-D)...\x1b[0m");
                break;
            }
            Err(err) => {
                eprintln!("\x1b[31m[Readline Error]: {:?}\x1b[0m", err);
                break;
            }
        }
    }

    // Simpan history sebelum keluar
    let _ = rl.save_history(history_path);
    Ok(())
}
