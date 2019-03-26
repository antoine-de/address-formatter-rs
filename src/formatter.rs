use crate::Address;
use failure::Error;
use include_dir::{include_dir, include_dir_impl, Dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
struct Replace(String, String);

#[derive(Debug, Default)]
struct Template {
    address_template: String,
    replace: Vec<Replace>,
    postformat_replace: Vec<Replace>,
    change_country: Option<String>,
    use_country: Option<String>,
}

#[derive(Debug)]
struct Templates {
    default_template: Template,
    fallback_template: Template,
    templates_by_country: HashMap<String, Template>, // todo use a `CountryCode` wrapper as key?
}

pub struct Formatter {
    components: Vec<String>,
    component_aliases: HashMap<String, String>,
    templates: Templates,
    // state_codes: Vec<>,
    // country_to_lang: Vec<>,
    // county_codes: Vec<>,
    // abbreviations: Vec<>,
    // valid_replacement_components: Vec<>
}

#[derive(Default, Debug)]
pub struct Configuration {
    country_code: Option<String>,
    abbreviate: Option<bool>,
}

impl Formatter {
    pub fn default() -> Self {
        // read all the opencage configuration
        let opencage_dir = include_dir!("./address-formatting/conf");
        let component_file = include_str!("../address-formatting/conf/components.yaml");
        let templates_file = include_str!("../address-formatting/conf/countries/worldwide.yaml");
        let raw_components = yaml_rust::YamlLoader::load_from_str(component_file)
            .expect("impossible to read components.yaml file");

        let components = raw_components
            .iter()
            .map(|v| {
                v["name"]
                    .clone()
                    .into_string()
                    .expect("no name for component")
            })
            .collect();

        let mut component_aliases = HashMap::<String, String>::new();
        for c in &raw_components {
            if let Some(aliases) = c["aliases"].as_vec() {
                for a in aliases {
                    component_aliases.insert(
                        a.as_str().unwrap().to_string(),
                        c["name"].as_str().unwrap().to_string(),
                    );
                }
            }
        }

        let raw_templates = yaml_rust::YamlLoader::load_from_str(templates_file)
            .expect("impossible to read worldwide.yaml file");

        // let raw_templates = &raw_templates[0];

        let default_template = raw_templates[0]["default"]["address_template"]
            .as_str()
            .map(|t| Template {
                address_template: t.to_string(),
                ..Default::default()
            })
            .expect("no default address_template provided");
        let fallback_template = raw_templates[0]["default"]["fallback_template"]
            .as_str()
            .map(|t| Template {
                address_template: t.to_string(),
                ..Default::default()
            })
            .expect("no default address_template provided");

        let templates_by_country = raw_templates[0]
        .clone()
            .into_hash().unwrap()
            .into_iter()
            .filter(|(k, _v)| k.as_str().unwrap().to_string().len() == 2) // TODO wrap it in countrycode
            .map(|(k, v)| {
                let country_code = k.as_str().unwrap().to_string();
                let template = Template {
                    address_template: v["address_template"].as_str().unwrap_or("bob").to_string(),
                    ..Default::default()
                };
                (country_code, template)
            })
            .collect();
        let templates = Templates {
            default_template,
            fallback_template,
            templates_by_country,
        };
        println!("components: {:?}", &components);
        println!("aliases: {:?}", &component_aliases);
        println!("templates: {:?}", &templates);
        Self {
            components,
            component_aliases,
            templates
        }
    }

    pub fn format(&self, addr: &Address) -> Result<String, Error> {
        let country_code = self.find_country_code(addr, Configuration::default());
        unimplemented!()
    }

    pub fn format_with_config(&self, addr: &Address, conf: Configuration) -> Result<String, Error> {
        let country_code = self.find_country_code(addr, conf);
        unimplemented!()
    }

    /// make an international one line label for the address
    // pub fn one_line_label(&self, addr: &Address, conf: Configuration) -> String {
    //     unimplemented!
    // }

    // /// make an international multi line label for the address
    // pub fn multi_line_label(&self, addr: &Address, conf: Configuration) -> String {
    //     unimplemented!
    // }

    fn find_country_code(&self, addr: &Address, conf: Configuration) -> Option<String> {
        unimplemented!()
    }
}
