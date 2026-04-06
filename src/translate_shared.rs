use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct LlamaResponse {
    choices: Vec<Choice>,
}
#[derive(Deserialize)]
struct Choice {
    message: Message,
}
#[derive(Deserialize)]
struct Message {
    content: String,
}
#[derive(Deserialize)]
struct TranslationResult {
    name: Option<String>,
    description: Option<String>,
    food_type: Option<String>,
    ingredients: Option<String>,
    alternate_names: Option<String>,
    labels: Option<String>,
}

pub struct TranslationOutput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub food_type: Option<String>,
    pub ingredients: Option<String>,
    pub alternate_names_pt_br: Option<Vec<String>>,
    pub labels_pt_br: Option<Vec<String>>,
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
    let mut input_map = HashMap::new();
    input_map.insert("name".to_string(), name.clone());
    if let Some(ref v) = description  { input_map.insert("description".to_string(),    v.clone()); }
    if let Some(ref v) = food_type    { input_map.insert("food_type".to_string(),       v.clone()); }
    if let Some(ref v) = ingredients  { input_map.insert("ingredients".to_string(),     v.clone()); }
    if let Some(ref v) = alternate_names { input_map.insert("alternate_names".to_string(), v.join(", ")); }
    if let Some(ref v) = labels       { input_map.insert("labels".to_string(),          v.join(", ")); }

    let input_json = sonic_rs::to_string(&input_map).unwrap_or_default();
    let prompt = format!(
        "Translate the strings of the following JSON from English to Brazilian Portuguese (pt-BR). \
         Strictly return ONLY a valid JSON object with the exact same keys.\n\nJSON:\n{}",
        input_json
    );
    let prompt_escaped = sonic_rs::to_string(&prompt).unwrap_or_else(|_| "\"\"".to_string());
    let body = format!(
        r#"{{"messages":[{{"role":"system","content":"/no_think You are an automated translator. Translate the values of the given JSON object to Portuguese pt-BR. Respond with ONLY valid JSON, no markdown formatting."}},{{"role":"user","content":{}}}],"temperature":0.1,"max_tokens":512,"stream":false}}"#,
        prompt_escaped
    );

    if let Ok(res) = client
        .post("http://127.0.0.1:8083/v1/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()
    {
        if let Ok(text) = res.text() {
            if let Ok(json_res) = sonic_rs::from_str::<LlamaResponse>(&text) {
                if let Some(choice) = json_res.choices.first() {
                    let cleaned = choice.message.content
                        .replace("```json", "").replace("```", "").trim().to_string();
                    if let Ok(t) = sonic_rs::from_str::<TranslationResult>(&cleaned) {
                        return TranslationOutput {
                            name: t.name,
                            description: t.description,
                            food_type: t.food_type,
                            ingredients: t.ingredients,
                            alternate_names_pt_br: t.alternate_names
                                .map(|s| s.split(", ").map(|x| x.to_string()).collect()),
                            labels_pt_br: t.labels
                                .map(|s| s.split(", ").map(|x| x.to_string()).collect()),
                        };
                    }
                }
            }
        }
    }

    TranslationOutput {
        name: None, description: None, food_type: None,
        ingredients: None, alternate_names_pt_br: None, labels_pt_br: None,
    }
}
