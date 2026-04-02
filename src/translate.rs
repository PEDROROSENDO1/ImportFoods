use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::collections::HashMap;
use crate::models::CleanFood;
use std::fs::OpenOptions;
use std::io::Write;

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

pub fn translate_batch(client: &Client, foods: &mut [CleanFood]) {
    std::thread::scope(|s| {
        for food in foods.iter_mut() {
            s.spawn(|| {
                translate_food(client, food);
            });
        }
    });
}

fn translate_food(client: &Client, food: &mut CleanFood) {
    let mut tasks: Vec<(&str, String)> = Vec::new();
    tasks.push(("name", food.name.clone()));
    if let Some(desc) = &food.description {
        tasks.push(("description", desc.clone()));
    }
    if let Some(ft) = &food.food_type {
        tasks.push(("food_type", ft.clone()));
    }
    if let Some(ing) = &food.ingredients {
        tasks.push(("ingredients", ing.clone()));
    }
    if let Some(alt) = &food.alternate_names {
        tasks.push(("alternate_names", alt.join(", ")));
    }
    if let Some(lab) = &food.labels {
        tasks.push(("labels", lab.join(", ")));
    }

    if tasks.is_empty() { return; }

    let mut input_map = HashMap::new();
    for (k, v) in &tasks {
        input_map.insert((*k).to_string(), v.to_string());
    }
    
    let input_json_str = sonic_rs::to_string(&input_map).unwrap_or_default();

    let prompt = format!("Translate the strings of the following JSON from English to Brazilian Portuguese (pt-BR). Strictly return ONLY a valid JSON object with the exact same keys.\n\nJSON:\n{}", input_json_str);
    let prompt_escaped = sonic_rs::to_string(&prompt).unwrap_or_else(|_| "\"\"".to_string());

    let req_body_str = format!(
        r#"{{"messages": [{{"role": "system", "content": "/no_think You are an automated translator. Translate the values of the given JSON object to Portuguese pt-BR. Respond with ONLY valid JSON, no markdown formatting."}}, {{"role": "user", "content": {}}}], "temperature": 0.1, "max_tokens": 512, "stream": false}}"#,
        prompt_escaped
    );

    if let Ok(res) = client.post("http://127.0.0.1:8083/v1/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .body(req_body_str)
        .send() {
        
        if let Ok(text) = res.text() {
            let ai_response = text.clone();
            let mut parsed_ok = false;
            if let Ok(json_res) = sonic_rs::from_str::<LlamaResponse>(&text) {
                if let Some(choice) = json_res.choices.first() {
                    let content = &choice.message.content;
                    let cleaned_content = content.replace("```json", "").replace("```", "").trim().to_string();
                    if let Ok(translated_json) = sonic_rs::from_str::<TranslationResult>(&cleaned_content) {
                        parsed_ok = true;
                        if let Some(val) = translated_json.name {
                            food.name_pt_br = Some(val);
                        }
                        if let Some(val) = translated_json.description {
                            food.description_pt_br = Some(val);
                        }
                        if let Some(val) = translated_json.food_type {
                            food.food_type_pt_br = Some(val);
                        }
                        if let Some(val) = translated_json.ingredients {
                            food.ingredients_pt_br = Some(val);
                        }
                        if let Some(val) = translated_json.alternate_names {
                            food.alternate_names_pt_br = Some(val.split(", ").map(|s| s.to_string()).collect());
                        }
                        if let Some(val) = translated_json.labels {
                            food.labels_pt_br = Some(val.split(", ").map(|s| s.to_string()).collect());
                        }
                    }
                }
            }
            
            // Log to file
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("translation_debug.log") {
                let _ = writeln!(
                    file,
                    "=== NEW IA REQUEST ===\nSENDING TO IA:\n{}\n\nRECEIVED FROM IA:\n{}\n\nPARSED OK: {}\n-------------------------------------------------------------",
                    input_json_str,
                    ai_response,
                    parsed_ok
                );
            }
        }
    }
}
