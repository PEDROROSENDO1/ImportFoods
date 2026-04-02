use sonic_rs::{from_str, JsonContainerTrait, JsonValueTrait, Value};
use std::collections::HashMap;
use std::{fs, io};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
pub struct RawFood {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub food_type: Option<String>,
    pub ean_13: Option<String>,
    pub alternate_names: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub ingredients: Option<String>,
    pub serving_metric_g: Option<f64>,
    pub serving_common_unit: Option<String>,
    pub serving_common_qty: Option<f64>,
    pub nutrients: HashMap<String, f64>,
}

#[derive(Default)]
pub struct Stats {
    pub total: usize,
    pub accepted: usize,
}

pub fn read_opennutrition_tsv(path: &str) -> Result<Vec<RawFood>, Box<dyn std::error::Error>> {
    println!("📂 Abrindo OpenNutrition TSV: {}", path);

    let file = fs::File::open(path)?;
    let file_size = file.metadata()?.len();

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(io::BufReader::new(file));

    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )?
        .progress_chars("#>-"),
    );

    let mut foods = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let pos = record.position().map(|p| p.byte()).unwrap_or(0);
        pb.set_position(pos);

        // Campos por índice conforme cabeçalho:
        // id, name, alternate_names, description, type, source, serving, nutrition_100g,
        // ean_13, labels, package_size, ingredients, ingredient_analysis
        let id = record.get(0).unwrap_or("").to_string();
        let name = record.get(1).unwrap_or("").to_string();

        if id.is_empty() || name.is_empty() {
            continue;
        }

        let alternate_names: Option<Vec<String>> = record
            .get(2)
            .and_then(|s| from_str(s).ok());

        let description = record.get(3).filter(|s| !s.is_empty()).map(|s| s.to_string());
        let food_type = record.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string());

        // serving: {"common":{"unit":"oz","quantity":3},"metric":{"unit":"g","quantity":85}}
        let (serving_metric_g, serving_common_unit, serving_common_qty) =
            if let Some(s) = record.get(6).filter(|s| !s.is_empty() && *s != "{}") {
                if let Ok(v) = from_str::<Value>(s) {
                    let metric_g = v.get("metric").and_then(|m: &Value| m.get("quantity")).and_then(|q: &Value| q.as_f64());
                    let common_unit = v.get("common").and_then(|c: &Value| c.get("unit")).and_then(|u: &Value| u.as_str()).map(|s: &str| s.to_string());
                    let common_qty = v.get("common").and_then(|c: &Value| c.get("quantity")).and_then(|q: &Value| q.as_f64());
                    (metric_g, common_unit, common_qty)
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

        // nutrition_100g
        let nutrients: HashMap<String, f64> =
            if let Some(s) = record.get(7).filter(|s| !s.is_empty() && *s != "{}") {
                if let Ok(v) = from_str::<Value>(s) {
                    if let Some(map) = v.as_object() {
                        map.iter()
                            .filter_map(|(k, v): (&str, &Value)| v.as_f64().map(|f| (k.to_string(), f)))
                            .collect()
                    } else {
                        HashMap::new()
                    }
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };

        let ean_13 = record.get(8).filter(|s| !s.is_empty()).map(|s| s.to_string());

        let labels: Option<Vec<String>> = record
            .get(9)
            .and_then(|s| from_str(s).ok())
            .filter(|v: &Vec<String>| !v.is_empty());

        let ingredients = record.get(11).filter(|s| !s.is_empty()).map(|s| s.to_string());

        foods.push(RawFood {
            id,
            name,
            description,
            food_type,
            ean_13,
            alternate_names,
            labels,
            ingredients,
            serving_metric_g,
            serving_common_unit,
            serving_common_qty,
            nutrients,
        });
    }

    pb.finish_with_message(format!("Leitura concluída: {} registros", foods.len()));
    Ok(foods)
}
