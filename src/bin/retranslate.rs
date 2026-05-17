#[path = "../translate_shared.rs"]
mod translate_shared;

use translate_shared::{translate_food_fields, translate_batch_opus, FoodRowInput, TranslationOutput};

use reqwest::blocking::Client;
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::remote::http::{Client as HttpClient, Http};
use surrealdb::opt::auth::Root;
use surrealdb::types::RecordId;
use surrealdb::types::SurrealValue;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

// ── structs ───────────────────────────────────────────────────────────────────

#[derive(SurrealValue, Deserialize, Debug, Clone)]
struct FoodRow {
    id: RecordId,
    name: String,
    description: Option<String>,
    food_type: Option<String>,
    ingredients: Option<String>,
    alternate_names: Option<Vec<String>>,
    labels: Option<Vec<String>>,
}

#[derive(SurrealValue, serde::Serialize)]
struct FoodUpdate {
    name_pt_br: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    food_type_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ingredients_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alternate_names_pt_br: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels_pt_br: Option<Vec<String>>,
}

// ── main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ws_url = "127.0.0.1:8000".to_string();
    let mut ns     = "main".to_string();
    let mut db     = "main".to_string();
    let mut user   = "root".to_string();
    let mut pass   = "root".to_string();
    let mut batch  = 50usize;
    let mut llm_slots = 4usize;
    let mut bench = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--url"   | "-u" => { if let Some(v) = args.next() { ws_url = v; } }
            "--ns"           => { if let Some(v) = args.next() { ns = v; } }
            "--db"           => { if let Some(v) = args.next() { db = v; } }
            "--user"         => { if let Some(v) = args.next() { user = v; } }
            "--pass"         => { if let Some(v) = args.next() { pass = v; } }
            "--batch" | "-b" => { if let Some(v) = args.next() { batch = v.parse().unwrap_or(50); } }
            "--llm-slots" | "-s" => { if let Some(v) = args.next() { llm_slots = v.parse().unwrap_or(4); } }
            "--bench" => { bench = true; }
            _ => {}
        }
    }

    if bench {
        println!("🔬 Rodando benchmark em todas as instâncias...");
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        let ports = [8084u16, 8085, 8086, 8087, 8088, 8089];
        let total_start = std::time::Instant::now();
        let results: Vec<_> = std::thread::scope(|s| {
            ports.iter().map(|&port| {
                let http = http.clone();
                s.spawn(move || {
                    let url = format!("http://127.0.0.1:{}/bench", port);
                    match http.get(&url).send().and_then(|r| r.text()) {
                        Ok(r) => format!("porta {}: {}", port, r),
                        Err(e) => format!("porta {}: offline ({})", port, e),
                    }
                })
            }).collect::<Vec<_>>()
            .into_iter().map(|h| h.join().unwrap()).collect()
        });
        for r in &results {
            println!("  {}", r);
        }
        println!("⏱️  Tempo total: {:.2}s", total_start.elapsed().as_secs_f64());
        return Ok(());
    }

    println!("🔌 Conectando ao SurrealDB em {}...", ws_url);
    let surreal: Arc<Surreal<HttpClient>> = Arc::new(
        tokio::time::timeout(Duration::from_secs(10), Surreal::new::<Http>(ws_url.as_str()))
            .await
            .map_err(|_| "Timeout ao conectar no SurrealDB")??
    );
    tokio::time::timeout(Duration::from_secs(10), surreal.signin(Root { username: user.clone(), password: pass.clone() }))
        .await
        .map_err(|_| "Timeout no signin")??;
    tokio::time::timeout(Duration::from_secs(10), surreal.use_ns(&ns).use_db(&db))
        .await
        .map_err(|_| "Timeout no use_ns/use_db")??;
    println!("✅ Conectado!");

    // ── conta total ───────────────────────────────────────────────────────────
    #[derive(Deserialize, SurrealValue)]
    struct CountRow { count: u64 }

    let mut count_res = surreal
        .query("SELECT count() FROM foods WHERE name_pt_br = NONE GROUP ALL;")
        .await?;
    let count_rows: Vec<CountRow> = count_res.take(0)?;
    let total = count_rows.first().map(|r| r.count).unwrap_or(0);

    println!("📊 Alimentos sem tradução: {}", total);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | {per_sec} | ETA: {eta}"
        ).unwrap()
    );
    pb.enable_steady_tick(Duration::from_millis(120));

    // ── health check a cada 20s ───────────────────────────────────────────────
    {
        let health_db = Arc::clone(&surreal);
        let health_pb = pb.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(20));
            loop {
                ticker.tick().await;
                match health_db.query("INFO FOR DB;").await {
                    Ok(_)  => health_pb.println("💚 [health] conexão OK"),
                    Err(e) => health_pb.println(format!("🔴 [health] falha na conexão: {}", e)),
                }
            }
        });
    }

    let http = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    // ── loop principal ────────────────────────────────────────────────────────
    // Não usa offset: cada lote traduzido sai do WHERE name_pt_br = NONE
    loop {
        let mut res = surreal
            .query(
                "SELECT id, name, description, food_type, ingredients, alternate_names, labels \
                 FROM foods WHERE name_pt_br = NONE LIMIT $batch;"
            )
            .bind(("batch", batch))
            .await?;

        let rows: Vec<FoodRow> = res.take(0)?;
        if rows.is_empty() { break; }
        let count = rows.len();

        // Tradução: manda todos os campos do batch numa única chamada ao OPUS
        let translated: Vec<TranslationOutput> = tokio::task::spawn_blocking({
            let rows  = rows.clone();
            let http  = http.clone();
            move || {
                let inputs: Vec<FoodRowInput> = rows.iter().map(|r| FoodRowInput {
                    name: r.name.clone(),
                    description: r.description.clone(),
                    food_type: r.food_type.clone(),
                    ingredients: r.ingredients.clone(),
                    alternate_names: r.alternate_names.clone(),
                    labels: r.labels.clone(),
                }).collect();
                translate_batch_opus(&http, &inputs)
            }
        }).await?;

        // UPDATE via SDK — sem string interpolation, sem risco de injeção
        for (row, t) in rows.iter().zip(translated.iter()) {
            // Se a IA não retornou nome, loga e pula — não salva inglês como pt_br
            let name_pt_br = match &t.name {
                Some(n) => n.clone(),
                None => {
                    pb.println(format!("⚠️  IA não traduziu '{}' — pulando para retentar depois", row.name));
                    continue;
                }
            };

            let update = FoodUpdate {
                name_pt_br,
                description_pt_br:     t.description.clone(),
                food_type_pt_br:       t.food_type.clone(),
                ingredients_pt_br:     t.ingredients.clone(),
                alternate_names_pt_br: t.alternate_names_pt_br.clone(),
                labels_pt_br:          t.labels_pt_br.clone(),
            };

            if let Err(e) = surreal
                .update::<Option<surrealdb::types::Value>>(row.id.clone())
                .merge(update)
                .await
            {
                pb.println(format!("⚠️  UPDATE falhou para {:?}: {}", row.id, e));
            }
        }

        pb.inc(count as u64);
        if count < batch { break; }
    }

    pb.finish_with_message("✅ Retradução concluída!");
    Ok(())
}
