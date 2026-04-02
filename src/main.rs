mod clean;
mod learning;
mod models;
mod translate;

use reqwest::blocking::{Client, Response};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue, SET_COOKIE};
use sonic_rs::{json, JsonValueTrait};
use std::time::Duration;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs::{self, OpenOptions};
use std::io::Write;

fn load_progress() -> Option<(String, usize)> {
    if let Ok(content) = fs::read_to_string("progress.json") {
        if let Ok(val) = sonic_rs::from_str::<sonic_rs::Value>(&content) {
            if let (Some(fp), Some(ci)) = (val["file_path"].as_str(), val["chunk_index"].as_u64()) {
                return Some((fp.to_string(), ci as usize));
            }
        }
    }
    None
}

fn save_progress(file_path: &str, chunk_index: usize) {
    if let Ok(content) = sonic_rs::to_string(&json!({
        "file_path": file_path,
        "chunk_index": chunk_index
    })) {
        let _ = fs::write("progress.json", content);
    }
}

fn clear_progress() {
    let _ = fs::remove_file("progress.json");
}

fn login(client: &Client, api_base: &str, username: &str, password: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let url = format!("{}/api/auth/login", api_base);
    let body_str = sonic_rs::to_string(&json!({
        "username": username,
        "password": password,
        "captcha_id": "",
        "captcha_answer": ""
    })).unwrap();

    let resp: Response = client.post(&url)
        .header("Content-Type", "application/json")
        .body(body_str)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("Login falhou ({}): {}", status, text).into());
    }

    // Extrair cookie antes de consumir o body
    let cookie_header = resp
        .headers()
        .get(SET_COOKIE)
        .ok_or("Cookie de sessão não encontrado na resposta")?
        .to_str()?
        .to_string();

    let session_cookie = cookie_header
        .split(';')
        .next()
        .ok_or("Cookie malformado")?
        .trim()
        .to_string();

    let json_text = resp.text()?;
    let json_body: sonic_rs::Value = sonic_rs::from_str(&json_text).map_err(|e| format!("JSON error: {}", e))?;
    let csrf_token = json_body["csrf_token"]
        .as_str()
        .ok_or("csrf_token não encontrado na resposta de login")?
        .to_string();

    Ok((session_cookie, csrf_token))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api_base = "http://192.168.15.8:8080".to_string();
    let mut files = Vec::new();
    let mut username = String::new();
    let mut password = String::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--api-url" | "-a" => {
                if let Some(val) = args.next() { api_base = val; }
            }
            "--username" | "-u" => {
                if let Some(val) = args.next() { username = val; }
            }
            "--password" | "-p" => {
                if let Some(val) = args.next() { password = val; }
            }
            _ => files.push(arg),
        }
    }

    if username.is_empty() || password.is_empty() {
        eprintln!("Uso: import_foods -u <usuario> -p <senha> [--api-url <url>] [arquivo.tsv ...]");
        std::process::exit(1);
    }

    if files.is_empty() {
        files.push("data/opennutrition_foods.tsv".to_string());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("🔐 Fazendo login como '{}'...", username);
    let (session_cookie, csrf_token) = login(&client, &api_base, &username, &password)?;
    println!("✅ Login bem-sucedido!");

    let api_url = format!("{}/api/lifestyle/admin/foods/batch", api_base);

    println!("🚀 Iniciando Importação de Alimentos (OpenNutrition)");

    let progress = load_progress();
    if let Some((ref fp, ci)) = progress {
        println!("📂 Progresso encontrado: Arquivo {} no lote {}", fp, ci);
    }

    let (mp, overall_pb) = setup_progress_bars(files.len() as u64);

    let mut skip_files = true;

    for file_path in files {
        if let Some((ref fp, _)) = progress {
            if skip_files {
                if fp != &file_path {
                    println!("⏭️ Pulando arquivo já processado: {}", file_path);
                    overall_pb.inc(1);
                    continue;
                } else {
                    skip_files = false;
                }
            }
        }

        overall_pb.set_message(format!("Processando: {}", file_path));

        let raw_data = learning::read_opennutrition_tsv(&file_path)?;

        if raw_data.is_empty() {
            println!("⚠️ Nenhum dado lido de {}", file_path);
            overall_pb.inc(1);
            continue;
        }

        let (clean_data, stats) = clean::process_and_clean(raw_data);

        let pct = if stats.total > 0 { stats.accepted as f64 / stats.total as f64 * 100.0 } else { 0.0 };
        println!("✅ {} -> Aceitos: {}/{} ({:.1}%)", file_path, stats.accepted, stats.total, pct);

        let mut clean_data = clean_data;
        let mut chunks: Vec<_> = clean_data.chunks_mut(100).collect();
        let total_chunks = chunks.len();
        let upload_pb = mp.add(ProgressBar::new(total_chunks as u64));
        upload_pb.set_style(ProgressStyle::with_template(
            "{spinner:.cyan} Traduzindo & Enviando API [{bar:20.cyan/blue}] {pos}/{len} {msg}",
        )?);

        for (i, chunk) in chunks.iter_mut().enumerate() {
            // Verifica e pula se necessário pelo progresso salvo
            if let Some((ref fp, ci)) = progress {
                if fp == &file_path && i < ci {
                    continue;
                }
            }

            save_progress(&file_path, i);

            translate::translate_batch(&client, chunk);

            let foods_json: Vec<sonic_rs::Value> = chunk
                .iter()
                .map(|f| sonic_rs::to_value(f).unwrap())
                .collect();

            let payload_str = sonic_rs::to_string(&json!({
                "source": "OPEN_NUTRITIONS",
                "foods": foods_json
            })).unwrap();

            let mut req_headers = HeaderMap::new();
            req_headers.insert(COOKIE, HeaderValue::from_str(&session_cookie)?);
            req_headers.insert("x-csrf-token", HeaderValue::from_str(&csrf_token)?);
            req_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

            let req_body_log = payload_str.clone();
            
            match client.post(&api_url).headers(req_headers).body(payload_str).send() {
                Ok(response) if response.status().is_success() => {
                    let text = response.text().unwrap_or_default();
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("api_debug.log") {
                        let _ = writeln!(file, "=== NEW API REQUEST ===\nSENDING PAYLOAD:\n{}\n\nRECEIVED RESPONSE: [SUCCESS]\n{}\n-------------------------------------------------------------", req_body_log, text);
                    }
                    upload_pb.inc(1);
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().unwrap_or_default();
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("api_debug.log") {
                        let _ = writeln!(file, "=== NEW API REQUEST ===\nSENDING PAYLOAD:\n{}\n\nRECEIVED RESPONSE: [{}]\n{}\n-------------------------------------------------------------", req_body_log, status, body);
                    }
                    eprintln!("   Erro no lote {}/{}: Status {} - {}", i + 1, total_chunks, status, body);
                }
                Err(e) => {
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("api_debug.log") {
                        let _ = writeln!(file, "=== NEW API REQUEST ===\nSENDING PAYLOAD:\n{}\n\nRECEIVED RESPONSE: [ERROR]\n{}\n-------------------------------------------------------------", req_body_log, e);
                    }
                    eprintln!("   Falha de conexão no lote {}/{}: {}", i + 1, total_chunks, e);
                }
            }
        }
        upload_pb.finish_with_message(format!("Envio de {} lotes finalizado", total_chunks));
        clear_progress(); // Limpa progresso do arquivo após conclusão
        overall_pb.inc(1);
    }

    overall_pb.finish_with_message("🎉 Processamento completo!");
    Ok(())
}

fn setup_progress_bars(total_files: u64) -> (MultiProgress, ProgressBar) {
    let mp = MultiProgress::new();
    let overall_pb = mp.add(ProgressBar::new(total_files));
    overall_pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} Progresso Geral: [{bar:40.green/white}] {pos}/{len} arquivos ({percent}%) {msg}",
        )
        .expect("Falha no template da barra")
        .progress_chars("#--"),
    );
    (mp, overall_pb)
}
