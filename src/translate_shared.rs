use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

pub struct TranslationOutput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub food_type: Option<String>,
    pub ingredients: Option<String>,
    pub alternate_names_pt_br: Option<Vec<String>>,
    pub labels_pt_br: Option<Vec<String>>,
}

// Chama o servidor OPUS-MT com um batch de textos, retorna traduções na mesma ordem.
static OPUS_PORT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
const OPUS_PORTS: &[u16] = &[8084, 8085, 8086, 8087, 8088, 8089];

fn call_opus(client: &Client, texts: &[&str]) -> Vec<Option<String>> {
    let idx = OPUS_PORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % OPUS_PORTS.len();
    let port = OPUS_PORTS[idx];
    let url = format!("http://127.0.0.1:{}/translate", port);
    let body = match sonic_rs::to_string(&serde_json::json!({ "texts": texts })) {
        Ok(b) => b,
        Err(_) => return texts.iter().map(|_| None).collect(),
    };

    let text = match client
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .and_then(|r| r.text())
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ [OPUS] HTTP error: {}", e);
            return texts.iter().map(|_| None).collect();
        }
    };

    match sonic_rs::from_str::<Vec<String>>(&text) {
        Ok(v) if v.len() == texts.len() => v.into_iter().map(Some).collect(),
        Ok(v) => {
            eprintln!("❌ [OPUS] retornou {} itens, esperava {}", v.len(), texts.len());
            texts.iter().map(|_| None).collect()
        }
        Err(e) => {
            eprintln!("❌ [OPUS] parse falhou: {} | raw: {}", e, &text[..text.len().min(200)]);
            texts.iter().map(|_| None).collect()
        }
    }
}

pub fn translate_food_fields(
    client: &Client,
    name: String,
    description: Option<String>,
    food_type: Option<String>,
    ingredients: Option<String>,
    alternate_names: Option<Vec<String>>,
    labels: Option<Vec<String>>,
) -> TranslationOutput {
    // Monta lista de textos para traduzir em batch
    let alt_str = alternate_names.as_ref().map(|v| v.join(", "));
    let lab_str = labels.as_ref().map(|v| v.join(", "));

    let fields: Vec<Option<&str>> = vec![
        Some(name.as_str()),
        description.as_deref(),
        food_type.as_deref(),
        ingredients.as_deref(),
        alt_str.as_deref(),
        lab_str.as_deref(),
    ];

    // Filtra apenas os campos que existem, mantendo índice para restaurar
    let (indices, texts): (Vec<usize>, Vec<&str>) = fields
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|t| (i, t)))
        .unzip();

    if texts.is_empty() {
        return empty();
    }

    let translated = call_opus(client, &texts);

    // Reconstrói no índice original
    let mut result = vec![None::<String>; 6];
    for (idx, val) in indices.into_iter().zip(translated.into_iter()) {
        result[idx] = val;
    }

    TranslationOutput {
        name:                 result[0].take(),
        description:          result[1].take(),
        food_type:            result[2].take(),
        ingredients:          result[3].take(),
        alternate_names_pt_br: result[4].take()
            .map(|s| s.split(", ").map(|x| x.to_string()).collect()),
        labels_pt_br:         result[5].take()
            .map(|s| s.split(", ").map(|x| x.to_string()).collect()),
    }
}

fn empty() -> TranslationOutput {
    TranslationOutput {
        name: None, description: None, food_type: None,
        ingredients: None, alternate_names_pt_br: None, labels_pt_br: None,
    }
}

// Estrutura mínima para o batch — usada pelo retranslate
pub struct FoodRowInput {
    pub name: String,
    pub description: Option<String>,
    pub food_type: Option<String>,
    pub ingredients: Option<String>,
    pub alternate_names: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
}

// Traduz um batch inteiro numa única chamada HTTP ao OPUS-MT.
// Todos os campos de todos os alimentos vão num único array de textos.
pub fn translate_batch_opus(client: &Client, rows: &[FoodRowInput]) -> Vec<TranslationOutput> {
    // Monta (row_idx, field_idx, texto) para cada campo não-nulo
    let mut entries: Vec<(usize, usize, String)> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        entries.push((i, 0, row.name.clone()));
        if let Some(ref v) = row.description     { entries.push((i, 1, v.clone())); }
        if let Some(ref v) = row.food_type       { entries.push((i, 2, v.clone())); }
        if let Some(ref v) = row.ingredients     { entries.push((i, 3, v.clone())); }
        if let Some(ref v) = row.alternate_names { entries.push((i, 4, v.join(", "))); }
        if let Some(ref v) = row.labels          { entries.push((i, 5, v.join(", "))); }
    }

    if entries.is_empty() {
        return rows.iter().map(|_| empty()).collect();
    }

    let texts: Vec<&str> = entries.iter().map(|(_, _, t)| t.as_str()).collect();
    let translated = call_opus(client, &texts);

    // Reconstrói por alimento
    let mut results: Vec<[Option<String>; 6]> = rows.iter().map(|_| Default::default()).collect();
    for ((row_i, field_i, _), val) in entries.iter().zip(translated.into_iter()) {
        results[*row_i][*field_i] = val;
    }

    results.into_iter().map(|mut r| TranslationOutput {
        name:                 r[0].take(),
        description:          r[1].take(),
        food_type:            r[2].take(),
        ingredients:          r[3].take(),
        alternate_names_pt_br: r[4].take()
            .map(|s| s.split(", ").map(|x| x.to_string()).collect()),
        labels_pt_br:         r[5].take()
            .map(|s| s.split(", ").map(|x| x.to_string()).collect()),
    }).collect()
}
