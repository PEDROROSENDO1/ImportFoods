use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CleanFood {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_type_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ean_13: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_names_pt_br: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_pt_br: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingredients: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingredients_pt_br: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serving_metric_g: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serving_common_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serving_common_qty: Option<f64>,
    pub nutrients: Nutrients,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Nutrients {
    // Energia
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<f64>,

    // Macros
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carbohydrates: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_fat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dietary_fiber: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soluble_fiber: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insoluble_fiber: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_sugars: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_sugars: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_carbohydrates: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sugar_alcohols: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethyl_alcohol: Option<f64>,

    // Gorduras
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturated_fats: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monounsaturated_fats: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polyunsaturated_fats: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trans_fats: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cholesterol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omega_3: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omega_6: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omega_9: Option<f64>,

    // Ácidos graxos específicos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lauric_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub myristic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palmitic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stearic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oleic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linoleic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linolenic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha_linolenic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamma_linolenic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arachidonic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eicosapentaenoic_acid: Option<f64>,  // EPA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docosapentaenoic_acid: Option<f64>,  // DPA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docosahexaenoic_acid: Option<f64>,   // DHA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eicosenoic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erucic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capric_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caprylic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dihomo_gamma_linolenic_acid: Option<f64>,

    // Aminoácidos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alanine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arginine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspartic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cystine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cysteine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glutamic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glycine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histidine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isoleucine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leucine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lysine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methionine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phenylalanine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proline: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threonine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tryptophan: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyrosine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taurine: Option<f64>,

    // Minerais
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calcium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copper: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iron: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnesium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manganese: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phosphorus: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub potassium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sodium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zinc: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selenium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iodine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chlorine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chromium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub molybdenum: Option<f64>,

    // Vitaminas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_b6: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_b12: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_d: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_e: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vitamin_k: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thiamin: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub riboflavin: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub niacin: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pantothenic_acid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folate_dfe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biotin: Option<f64>,

    // Outros
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caffeine: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choline: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorbitol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xylitol: Option<f64>,
}
