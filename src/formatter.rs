use crate::Address;
use failure::Fail;
use failure::{format_err, Error};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct Replace(String, String);

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct CountryCode(String); // TODO small string

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

    pub fn format(&self, mut addr: Address) -> Result<String, Error> {
        self.format_with_config(addr, Configuration::default())
    }

    pub fn format_with_config(
        &self,
        mut addr: Address,
        conf: Configuration,
    ) -> Result<String, Error> {
        let country_code = self.find_country_code(&addr, conf);

        self.alias_fields(&mut addr);

        sanity_clean_address(&mut addr);

        let template = self.find_template(&addr, country_code);

        let template_engine = crate::handlebar_helper::new_template_engine();

        println!("template {}", &template.address_template);

        let mut text = template_engine
            .render_template(&template.address_template, &addr)
            .map_err(|e| e.context("impossible to render template"))?;

        println!("text == {}", &text);

        Ok(text)
    }

    /// make an international one line label for the address
    // pub fn one_line_label(&self, addr: &Address, conf: Configuration) -> String {
    //     unimplemented!
    // }

    // /// make an international multi line label for the address
    // pub fn multi_line_label(&self, addr: &Address, conf: Configuration) -> String {
    //     unimplemented!
    // }

    fn find_country_code(&self, addr: &Address, conf: Configuration) -> Option<CountryCode> {
        conf.country_code
            .or(addr.country_code.clone())
            .and_then(|s| {
                CountryCode::from_str(&s)
                    .map_err(|e| log::info!("impossible to find a country: {}", e))
                    .ok()
            })
    }

    fn alias_fields(&self, addr: &mut Address) {
        // TODO use the aliases
        /*

        foreach ($this->componentAliases as $key => $val) {
            if (isset($addressArray[$key]) && !isset($addressArray[$val])) {
                $addressArray[$val] = $addressArray[$key];
            }
        }

        */
    }

    fn find_template<'a>(
        &'a self,
        addr: &Address,
        country_code: Option<CountryCode>,
    ) -> &'a Template {
        country_code
            .and_then(|c| {
                if !has_minimum_address_components(addr) {
                    Some(&self.templates.fallback_template)
                } else {
                    self.templates.templates_by_country.get(&c)
                }
            })
            .unwrap_or(&self.templates.default_template)
    }
}

fn sanity_clean_address(addr: &mut Address) {
    //TODO cleanup data
    /*
        if (isset($addressArray['postcode'])) {
            if (strlen($addressArray['postcode']) > 20) {
                unset($addressArray['postcode']);
            } elseif (preg_match('/\d+;\d+/', $addressArray['postcode']) > 0) {
                // Sometimes OSM has postcode ranges
                unset($addressArray['postcode']);
            } elseif (preg_match('/^(\d{5}),\d{5}/', $addressArray['postcode'], $matches) > 0) {
                // Use the first postcode from the range
                $addressArray['postcode'] = $matches[1];
            }
        }
        //Try and catch values containing URLs
        foreach ($addressArray as $key => $val) {
            if (preg_match('|https?://|', $val) > 0) {
                unset($addressArray[$key]);
            }
        }



    /**
     * Hacks for bad country data
     */
    if (isset($addressArray['country'])) {
    if (isset($addressArray['state'])) {
    /**
                 * If the country is a number, use the state as country
                 */
                if (is_numeric($addressArray['country'])) {
                    $addressArray['country'] = $addressArray['state'];
                    unset($addressArray['state']);
                }
            }
        }
    */
}

fn has_minimum_address_components(addr: &Address) -> bool {
    //TODO
    true
}
