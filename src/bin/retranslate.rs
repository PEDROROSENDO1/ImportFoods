#[path = "../translate_shared.rs"]
mod translate_shared;

use translate_shared::{translate_food_fields, TranslationOutput};

use reqwest::blocking::Client;
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client as WsClient, Ws};
use surrealdb::opt::auth::Root;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

// ── structs ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
struct FoodRow {
    id: surrealdb::RecordId,
    name: String,
    description: Option<String>,
    food_type: Option<String>,
    ingredients: Option<String>,
    alternate_names: Option<Vec<String>>,
    labels: Option<Vec<String>>,
}

#[derive(serde::Serialize)]
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
    let mut ws_url = "ws://127.0.0.1:8000".to_string();
    let mut ns     = "main".to_string();
    let mut db     = "main".to_string();
    let mut user   = "root".to_string();
    let mut pass   = "root".to_string();
    let mut batch  = 50usize;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--url"   | "-u" => { if let Some(v) = args.next() { ws_url = v; } }
            "--ns"           => { if let Some(v) = args.next() { ns = v; } }
            "--db"           => { if let Some(v) = args.next() { db = v; } }
            "--user"         => { if let Some(v) = args.next() { user = v; } }
            "--pass"         => { if let Some(v) = args.next() { pass = v; } }
            "--batch" | "-b" => { if let Some(v) = args.next() { batch = v.parse().unwrap_or(50); } }
            _ => {}
        }
    }

    println!("🔌 Conectando ao SurrealDB em {}...", ws_url);
    let surreal: Arc<Surreal<WsClient>> = Arc::new(
        Surreal::new::<Ws>(ws_url.as_str()).await?
    );
    surreal.signin(Root { username: &user, password: &pass }).await?;
    surreal.use_ns(&ns).use_db(&db).await?;
    println!("✅ Conectado!");

    // ── health check a cada 20s ───────────────────────────────────────────────
    {
        let health_db = Arc::clone(&surreal);
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(20));
            loop {
                ticker.tick().await;
                match health_db.query("INFO FOR DB;").await {
                    Ok(_)  => println!("💚 [health] conexão OK"),
                    Err(e) => eprintln!("🔴 [health] falha na conexão: {}", e),
                }
            }
        });
    }

    // ── conta total ───────────────────────────────────────────────────────────
    let mut count_res = surreal
        .query("SELECT count() FROM foods WHERE name_pt_br = NONE GROUP ALL;")
        .await?;
    let count_rows: Vec<serde_json::Value> = count_res.take(0)?;
    let total = count_rows.first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("📊 Alimentos sem tradução: {}", total);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) ETA: {eta}"
        ).unwrap()
    );

    let http = Client::builder()
        .timeout(Duration::from_secs(60))
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

        // Tradução paralela em threads bloqueantes (fora do runtime async)
        let translated: Vec<TranslationOutput> = tokio::task::spawn_blocking({
            let rows  = rows.clone();
            let http  = http.clone();
            move || {
                std::thread::scope(|s| {
                    rows.iter()
                        .map(|row| s.spawn(|| translate_food_fields(
                            &http,
                            row.name.clone(),
                            row.description.clone(),
                            row.food_type.clone(),
                            row.ingredients.clone(),
                            row.alternate_names.clone(),
                            row.labels.clone(),
                        )))
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|h| h.join().unwrap())
                        .collect()
                })
            }
        }).await?;

        // UPDATE via SDK — sem string interpolation, sem risco de injeção
        for (row, t) in rows.iter().zip(translated.iter()) {
            // Se a IA não retornou nome, loga e pula — não salva inglês como pt_br
            let name_pt_br = match &t.name {
                Some(n) => n.clone(),
                None => {
                    eprintln!("⚠️  IA não traduziu '{}' — pulando para retentar depois", row.name);
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
                .update::<Option<serde_json::Value>>(row.id.clone())
                .merge(update)
                .await
            {
                eprintln!("⚠️  UPDATE falhou para {}: {}", row.id, e);
            }
        }

        pb.inc(count as u64);
        if count < batch { break; }
    }

    pb.finish_with_message("✅ Retradução concluída!");
    Ok(())
}
