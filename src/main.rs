use ethers::prelude::*;
use ethers::utils::to_checksum;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::task;
use num_cpus;
use std::io;

#[tokio::main]
async fn main() {
    let mode: i32 = loop {
        println!("Оберіть режим роботи:");
        println!("1 - Масова генерація гаманців");
        println!("2 - Генерація гаманця з підбором префікса");
        println!("3 - Генерація гаманця з підбором префікса та суфікса");

        let mut mode_input = String::new();
        io::stdin().read_line(&mut mode_input).expect("Не вдалося прочитати рядок");

        match mode_input.trim().parse::<i32>() {
            Ok(num) if num >= 1 && num <= 3 => break num, // Якщо введено число від 1 до 3, виходимо з циклу
            _ => println!("Будь ласка, введіть число від 1 до 3."), // Інакше виводимо помилку та продовжуємо цикл
        };
    };
    match mode {
        1 => {
            println!("Введіть кількість гаманців для генерації:");
            let mut num_wallets = String::new();
            io::stdin().read_line(&mut num_wallets).expect("Не вдалося прочитати рядок");
            let num_wallets: usize = num_wallets.trim().parse().expect("Введіть коректну кількість");

            generate_wallets(num_wallets).await;
        },
        2 => {
            println!("Введіть бажаний префікс адреси:");
            let mut desired_prefix = String::new();
            io::stdin().read_line(&mut desired_prefix).expect("Не вдалося прочитати рядок");
            let desired_prefix = desired_prefix.trim().to_string();

            find_wallet_with_prefix(desired_prefix).await;
        },
        3 => {
            println!("Введіть бажаний префікс адреси:");
            let mut desired_prefix = String::new();
            io::stdin().read_line(&mut desired_prefix).expect("Не вдалося прочитати рядок");
            let desired_prefix = desired_prefix.trim().to_string();

            println!("Введіть бажаний суфікс адреси:");
            let mut desired_suffix = String::new();
            io::stdin().read_line(&mut desired_suffix).expect("Не вдалося прочитати рядок");
            let desired_suffix = desired_suffix.trim().to_string();

            find_wallet_with_prefix_and_suffix(desired_prefix, desired_suffix).await;
        },
        _ => println!("Невідомий режим роботи"),
    }
}

async fn generate_wallets(num_wallets: usize) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("wallets.txt")
        .expect("Не вдалося відкрити файл");

    for _ in 0..num_wallets {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let address = wallet.address();
        let private_key_bytes = wallet.signer().to_bytes();
        let private_key_hex = format!("0x{}", private_key_bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());

        writeln!(file, "Address: {:?}, Private key: {}", address, private_key_hex)
            .expect("Не вдалося записати у файл");

        println!("Generated wallet - Address: {:?}, Private key: {}", address, private_key_hex);
    }

    println!("{} wallets were generated and saved to wallets.txt", num_wallets);
}


async fn find_wallet_with_prefix(desired_prefix: String) {
    let found = Arc::new(Mutex::new(false));
    let num_threads = num_cpus::get();
    println!("Використовується потоків: {}", num_threads);

    let handles = (0..num_threads).map(|_| {
        let found = Arc::clone(&found);
        let desired_prefix = desired_prefix.clone();
        task::spawn(async move {
            let mut rng = StdRng::from_entropy();
            let mut count = 0;
            let start = Instant::now();
            while !*found.lock().await {
                let wallet = LocalWallet::new(&mut rng);
                let address = wallet.address();
                let checksum_address = to_checksum(&address, None);
                count += 1;

                if count % 1000 == 0 {
                    let duration = start.elapsed().as_secs_f32();
                    let speed = count as f32 / duration;
                    println!("Speed: {:.2} addresses/sec", speed);
                }

                if checksum_address[2..(2 + desired_prefix.len())] == desired_prefix {
                    let mut found_guard = found.lock().await;
                    if !*found_guard {
                        *found_guard = true;
                        let mut file = OpenOptions::new().append(true).create(true).open("found_wallets.txt").expect("Не вдалося відкрити файл");
                        let private_key_bytes = wallet.signer().to_bytes();
                        let private_key_hex = format!("0x{}", private_key_bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
                        writeln!(file, "Address: {:?}, Private key: {:?}", checksum_address, private_key_hex).expect("Не вдалося записати у файл");
                        println!("Found address: {:?}", checksum_address);
                        println!("Private key: {:?}", private_key_hex);
                    }
                    break;
                }
            }
        })
    });

    futures::future::join_all(handles).await;
}

async fn find_wallet_with_prefix_and_suffix(desired_prefix: String, desired_suffix: String) {
    let found = Arc::new(Mutex::new(false));
    let num_threads = num_cpus::get();
    println!("Використовується потоків: {}", num_threads);

    let handles = (0..num_threads).map(|_| {
        let found = Arc::clone(&found);
        let desired_prefix = desired_prefix.clone();
        let desired_suffix = desired_suffix.clone();
        task::spawn(async move {
            let mut rng = StdRng::from_entropy();
            let mut count = 0;
            let start = Instant::now();
            while !*found.lock().await {
                let wallet = LocalWallet::new(&mut rng);
                let address = wallet.address();
                let checksum_address = to_checksum(&address, None);
                count += 1;

                if count % 1000 == 0 {
                    let duration = start.elapsed().as_secs_f32();
                    let speed = count as f32 / duration;
                    println!("Speed: {:.2} addresses/sec", speed);
                }

                if checksum_address[2..(2 + desired_prefix.len())] == desired_prefix &&
                   checksum_address.ends_with(&desired_suffix) {
                    let mut found_guard = found.lock().await;
                    if !*found_guard {
                        *found_guard = true;
                        let mut file = OpenOptions::new().append(true).create(true).open("found_wallets_with_prefix_suffix.txt").expect("Не вдалося відкрити файл");
                        let private_key_bytes = wallet.signer().to_bytes();
                        let private_key_hex = format!("0x{}", private_key_bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
                        writeln!(file, "Address: {:?}, Private key: {:?}", checksum_address, private_key_hex).expect("Не вдалося записати у файл");
                        println!("Found address: {:?}, Private key: {:?}", checksum_address, private_key_hex);
                    }
                    break;
                }
            }
        })
    });

    futures::future::join_all(handles).await;
}

