use crate::Address;
use failure::{format_err, Error};
use include_dir::{include_dir, include_dir_impl, Dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct Replace(String, String);

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct CountryCode(String); // TODO small string

impl FromStr for CountryCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 2 {
            if s == "UK" {
                Ok(CountryCode("GB".to_owned()))
            } else {
                Ok(CountryCode(s.to_uppercase()))
            }
        } else {
            Err(format_err!(
                "{} is not a valid ISO3166-1:alpha2 country code",
                s,
            ))
        }
    }
}

#[derive(Debug, Default, Clone)]
struct NewComponent {
component: String,
new_value: String,
}

impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
struct Template {
    address_template: String,
    replace: Vec<Replace>,
    postformat_replace: Vec<Replace>,
    change_country: Option<String>,
    add_component: Option<NewComponent>,
}

#[derive(Debug)]
struct Templates {
    default_template: Template,
    fallback_template: Template,
    templates_by_country: HashMap<CountryCode, Template>,
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
        // let opencage_dir = include_dir!("./address-formatting/conf");
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

        // some countries uses the same rules as other countries (with some slight changes)
        // they are marked as `use_country: another_country_code`
        // we store them separatly first, to be able to create fully built templates
        let mut overrided_countries = HashMap::new();

        let mut templates_by_country: HashMap<CountryCode, Template> = raw_templates[0]
            .clone()
            .into_hash()
            .unwrap()
            .into_iter()
            .filter_map(|(k, v)| {
                k.as_str()
                    .and_then(|k| CountryCode::from_str(k).ok())
                    .map(|c| (c, v))
            })
            .filter_map(|(country_code, v)| {
                println!(" country code: {:?}", country_code);
                if let Some(parent_country) = v["use_country"].as_str().and_then(|k| CountryCode::from_str(k).ok()) {
                    // we store it for later processing
                    overrided_countries.insert(country_code, (parent_country, v.clone()));
                    None
                } else {
                    let template = Template {
                        address_template: v["address_template"]
                            .as_str()
                            .expect(&format!(
                                "no address_template found for country {}",
                                country_code
                            ))
                            .to_string(),
                        //TODO replace & postformat
                        ..Default::default()
                    };
                    Some((country_code, template))
                }
            })
            .collect();

        for (country_code, (parent_country_code, template)) in overrided_countries.into_iter() {
            let mut overrided_template = templates_by_country[&parent_country_code].clone();

            overrided_template.change_country = template["change_country"].as_str().map(|s| s.to_string());
            if let Some(add_component) = template["add_component"].as_str() {
                let part: Vec<_> = add_component.split("=").collect();
                assert_eq!(part.len(), 2);
                overrided_template.add_component = Some(NewComponent {
                    component: part[0].to_owned(),
                    new_value: part[1].to_owned()
                })
            }
            templates_by_country.insert(country_code, overrided_template);
        }

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
            templates,
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
