use crate::Address;
use failure::{format_err, Error};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct Replace(String, String);

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) struct CountryCode(String); // TODO small string

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
pub(crate) struct NewComponent {
    pub component: String,
    pub new_value: String,
}

impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Template {
    pub address_template: String,
    pub replace: Vec<Replace>,
    pub postformat_replace: Vec<Replace>,
    pub change_country: Option<String>,
    pub add_component: Option<NewComponent>,
}

#[derive(Debug)]
pub(crate) struct Templates {
    pub default_template: Template,
    pub fallback_template: Template,
    pub templates_by_country: HashMap<CountryCode, Template>,
}

pub struct Formatter {
    pub(crate) components: Vec<String>,
    pub(crate) component_aliases: HashMap<String, String>,
    pub(crate) templates: Templates,
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
        crate::read_configuration::read_configuration()
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
