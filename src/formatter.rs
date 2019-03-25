use crate::Address;
use failure::Error;
use include_dir::{include_dir, include_dir_impl, Dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
struct Template {
    address_template: String,
    replace: Option<Vec<Vec<String>>>,
    postformat_replace: Option<Vec<Vec<String>>>,
    change_country: Option<String>,
    use_country: Option<String>,
}

#[derive(Debug)]
struct DefaultTemplate {
    address_template: Template,
    fallback_template: Template,
}

#[derive(Debug)]
struct Templates {
    default: DefaultTemplate,
    templates: HashMap<String, Template>, // todo use a `CountryCode` wrapper as key?
}

pub struct Formatter {
    components: Vec<String>,
    component_aliases: HashMap<String, String>,
    // templates: Templates,
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

        // let templates = serde_yaml::from_str(templates_file).unwrap();
        println!("components: {:?}", &components);
        println!("aliases: {:?}", &component_aliases);
        // println!("templates: {:?}", &templates);
        Self {
            components,
            component_aliases,
            // templates
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
