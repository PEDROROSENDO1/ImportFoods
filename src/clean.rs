use crate::learning::{RawFood, Stats};
use crate::models::{CleanFood, Nutrients};
use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

static RE_HTML_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());
static RE_MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" {2,}").unwrap());
static RE_ONLY_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[\d\s]+$").unwrap());

fn is_allowed_char(c: char) -> bool {
    c.is_alphabetic() || c.is_numeric() || matches!(c, ' ' | '.' | ',' | '-' | '%' | '&' | '(' | ')')
}

fn clean_name(raw: &str) -> Option<String> {
    if raw.trim().is_empty() { return None; }
    let s: String = raw.nfkc().collect();
    let s = RE_HTML_TAG.replace_all(&s, " ");
    let s: String = s.chars().map(|c| if is_allowed_char(c) { c } else { ' ' }).collect();
    let s = RE_MULTI_SPACE.replace_all(&s, " ").trim().to_string();
    if s.len() < 2 || s.len() > 200 || RE_ONLY_DIGITS.is_match(&s) { return None; }
    Some(s)
}

fn map_nutrients(raw_nutrients: HashMap<String, f64>) -> Nutrients {
    let mut n = Nutrients::default();
    for (k, v) in raw_nutrients {
        if v < 0.0 { continue; }
        match k.as_str() {
            "calories"                      => n.calories = Some(v),
            "protein"                       => n.protein = Some(v),
            "carbohydrates"                 => n.carbohydrates = Some(v),
            "total_fat"                     => n.total_fat = Some(v),
            "dietary_fiber"                 => n.dietary_fiber = Some(v),
            "soluble_fiber"                 => n.soluble_fiber = Some(v),
            "insoluble_fiber"               => n.insoluble_fiber = Some(v),
            "total_sugars"                  => n.total_sugars = Some(v),
            "added_sugars"                  => n.added_sugars = Some(v),
            "other_carbohydrates"           => n.other_carbohydrates = Some(v),
            "sugar_alcohols"                => n.sugar_alcohols = Some(v),
            "water"                         => n.water = Some(v),
            "ethyl_alcohol"                 => n.ethyl_alcohol = Some(v),
            "saturated_fats"                => n.saturated_fats = Some(v),
            "monounsaturated_fats"          => n.monounsaturated_fats = Some(v),
            "polyunsaturated_fats"          => n.polyunsaturated_fats = Some(v),
            "trans_fats"                    => n.trans_fats = Some(v),
            "cholesterol"                   => n.cholesterol = Some(v),
            "omega_3"                       => n.omega_3 = Some(v),
            "omega_6"                       => n.omega_6 = Some(v),
            "omega_9"                       => n.omega_9 = Some(v),
            "lauric_acid"                   => n.lauric_acid = Some(v),
            "myristic_acid"                 => n.myristic_acid = Some(v),
            "palmitic_acid"                 => n.palmitic_acid = Some(v),
            "stearic_acid"                  => n.stearic_acid = Some(v),
            "oleic_acid"                    => n.oleic_acid = Some(v),
            "linoleic_acid"                 => n.linoleic_acid = Some(v),
            "linolenic_acid"                => n.linolenic_acid = Some(v),
            "alpha_linolenic_acid"          => n.alpha_linolenic_acid = Some(v),
            "gamma_linolenic_acid"          => n.gamma_linolenic_acid = Some(v),
            "arachidonic_acid"              => n.arachidonic_acid = Some(v),
            "eicosapentaenoic_acid"         => n.eicosapentaenoic_acid = Some(v),
            "docosapentaenoic_acid"         => n.docosapentaenoic_acid = Some(v),
            "docosahexaenoic_acid"          => n.docosahexaenoic_acid = Some(v),
            "eicosenoic_acid"               => n.eicosenoic_acid = Some(v),
            "erucic_acid"                   => n.erucic_acid = Some(v),
            "capric_acid"                   => n.capric_acid = Some(v),
            "caprylic_acid"                 => n.caprylic_acid = Some(v),
            "dihomo_gamma_linolenic_acid"   => n.dihomo_gamma_linolenic_acid = Some(v),
            "alanine"                       => n.alanine = Some(v),
            "arginine"                      => n.arginine = Some(v),
            "aspartic_acid"                 => n.aspartic_acid = Some(v),
            "cystine"                       => n.cystine = Some(v),
            "cysteine"                      => n.cysteine = Some(v),
            "glutamic_acid"                 => n.glutamic_acid = Some(v),
            "glycine"                       => n.glycine = Some(v),
            "histidine"                     => n.histidine = Some(v),
            "isoleucine"                    => n.isoleucine = Some(v),
            "leucine"                       => n.leucine = Some(v),
            "lysine"                        => n.lysine = Some(v),
            "methionine"                    => n.methionine = Some(v),
            "phenylalanine"                 => n.phenylalanine = Some(v),
            "proline"                       => n.proline = Some(v),
            "serine"                        => n.serine = Some(v),
            "threonine"                     => n.threonine = Some(v),
            "tryptophan"                    => n.tryptophan = Some(v),
            "tyrosine"                      => n.tyrosine = Some(v),
            "valine"                        => n.valine = Some(v),
            "taurine"                       => n.taurine = Some(v),
            "calcium"                       => n.calcium = Some(v),
            "copper"                        => n.copper = Some(v),
            "iron"                          => n.iron = Some(v),
            "magnesium"                     => n.magnesium = Some(v),
            "manganese"                     => n.manganese = Some(v),
            "phosphorus"                    => n.phosphorus = Some(v),
            "potassium"                     => n.potassium = Some(v),
            "sodium"                        => n.sodium = Some(v),
            "zinc"                          => n.zinc = Some(v),
            "selenium"                      => n.selenium = Some(v),
            "iodine"                        => n.iodine = Some(v),
            "chlorine"                      => n.chlorine = Some(v),
            "chromium"                      => n.chromium = Some(v),
            "molybdenum"                    => n.molybdenum = Some(v),
            "vitamin_a"                     => n.vitamin_a = Some(v),
            "vitamin_b6"                    => n.vitamin_b6 = Some(v),
            "vitamin_b12"                   => n.vitamin_b12 = Some(v),
            "vitamin_c"                     => n.vitamin_c = Some(v),
            "vitamin_d"                     => n.vitamin_d = Some(v),
            "vitamin_e"                     => n.vitamin_e = Some(v),
            "vitamin_k"                     => n.vitamin_k = Some(v),
            "thiamin"                       => n.thiamin = Some(v),
            "riboflavin"                    => n.riboflavin = Some(v),
            "niacin"                        => n.niacin = Some(v),
            "pantothenic_acid"              => n.pantothenic_acid = Some(v),
            "folate_dfe"                    => n.folate_dfe = Some(v),
            "biotin"                        => n.biotin = Some(v),
            "caffeine"                      => n.caffeine = Some(v),
            "choline"                       => n.choline = Some(v),
            "sorbitol"                      => n.sorbitol = Some(v),
            "xylitol"                       => n.xylitol = Some(v),
            _ => {}
        }
    }
    n
}

use std::collections::HashMap;

fn clean_single_product(raw: RawFood) -> Option<CleanFood> {
    let name = clean_name(&raw.name)?;
    let nutrients = map_nutrients(raw.nutrients);

    Some(CleanFood {
        id: raw.id,
        name,
        name_pt_br: None,
        description: raw.description,
        description_pt_br: None,
        food_type: raw.food_type,
        food_type_pt_br: None,
        ean_13: raw.ean_13,
        alternate_names: raw.alternate_names,
        alternate_names_pt_br: None,
        labels: raw.labels,
        labels_pt_br: None,
        ingredients: raw.ingredients,
        ingredients_pt_br: None,
        serving_metric_g: raw.serving_metric_g,
        serving_common_unit: raw.serving_common_unit,
        serving_common_qty: raw.serving_common_qty,
        nutrients,
    })
}

pub fn process_and_clean(products: Vec<RawFood>) -> (Vec<CleanFood>, Stats) {
    let mut stats = Stats { total: products.len(), ..Default::default() };

    let clean_data: Vec<CleanFood> = products.into_iter().filter_map(|p| {
        let r = clean_single_product(p);
        if r.is_some() { stats.accepted += 1; }
        r
    }).collect();

    (clean_data, stats)
}
