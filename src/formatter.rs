use crate::{Address, Component};
use failure::Fail;
use failure::{format_err, Error};
use std::collections::HashMap;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub(crate) struct Replacement {
    pub regex: regex::Regex,
    pub replacement_value: String,
}

/// Replacement rule
/// a Replacement can be on all fields, or only one of them
#[derive(Debug, Clone)]
pub(crate) enum ReplaceRule {
    All(Replacement),
    Component((Component, Replacement)),
}

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

impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NewComponent {
    pub component: String,
    pub new_value: String,
}

/// The template represent the rules to apply to a `Address` to format it
#[derive(Debug, Default, Clone)]
pub(crate) struct Template {
    pub address_template: String,
    /// Moustache template
    pub replace: Vec<ReplaceRule>,
    pub postformat_replace: Vec<ReplaceRule>,
    pub change_country: Option<String>,
    /// Override the country
    pub add_component: Option<NewComponent>,
}

#[derive(Debug)]
pub(crate) struct Templates {
    pub default_template: Template,
    pub fallback_template: Template,
    pub templates_by_country: HashMap<CountryCode, Template>,
}

pub struct Formatter {
    pub(crate) components: Vec<String>, // TODO REMOVE ?
    pub(crate) component_aliases: HashMap<String, Component>,
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

    // TODO take an Into<Address> as parameter ?
    pub fn format(&self, addr: Address) -> Result<String, Error> {
        self.format_with_config(addr, Configuration::default())
    }

    pub fn format_with_config(
        &self,
        mut addr: Address,
        conf: Configuration,
    ) -> Result<String, Error> {
        let country_code = self.find_country_code(&addr, conf);

        sanity_clean_address(&mut addr);
        dbg!(&addr);

        let template = self.find_template(&addr, country_code);

        replace_before(&template, &mut addr);

        let template_engine = crate::handlebar_helper::new_template_engine();

        let text = template_engine
            .render_template(&template.address_template, &addr)
            .map_err(|e| e.context("impossible to render template"))?;

        dbg!(&text);
        let text = cleanup_rendered(&text);

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
            .or(addr[Component::CountryCode].clone())
            .and_then(|s| {
                CountryCode::from_str(&s)
                    .map_err(|e| log::info!("impossible to find a country: {}", e))
                    .ok()
            })
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

    pub fn build_address<'a>(
        &self,
        values: impl IntoIterator<Item = (&'a str, String)>,
    ) -> Address {
        //TODO move this outside the formatter ?
        let mut address = Address::default();
        for (k, v) in values.into_iter() {
            let component = Component::from_str(k)
                .ok()
                .or_else(|| self.component_aliases.get(k).cloned());
            if let Some(component) = component {
                address[component] = Some(v);
            }
        }
        address
    }
}

fn sanity_clean_address(_addr: &mut Address) {
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

fn has_minimum_address_components(_addr: &Address) -> bool {
    //TODO
    true
}

fn cleanup_rendered(text: &str) -> String {
    use itertools::Itertools;
    use regex::Regex;
    lazy_static::lazy_static! {
        static ref REPLACEMENTS:  [(Regex, &'static str); 12]= [
            (Regex::new(r#"[},\s]+$"#).unwrap(), ""),
            (Regex::new(r#"^[,\s]+"#).unwrap(), ""),
            (Regex::new(r#"^- "#).unwrap(), ""), // line starting with dash due to a parameter missing
            (Regex::new(r#",\s*,"#).unwrap(), ", "), //multiple commas to one
            (Regex::new(r#"[\t\p{Zs}]+,[\t\p{Zs}]+"#).unwrap(), ", "), //one horiz whitespace behind comma
            (Regex::new(r#"[\t\p{Zs}][\t\p{Zs}]+"#).unwrap(), " "), //multiple horiz whitespace to one
            (Regex::new(r#"[\t\p{Zs}]\n"#).unwrap(), "\n"), //horiz whitespace, newline to newline
            (Regex::new(r#"\n,"#).unwrap(), "\n"), //newline comma to just newline
            (Regex::new(r#",,+"#).unwrap(), ","), //multiple commas to one
            (Regex::new(r#",\n"#).unwrap(), "\n"), //comma newline to just newline
            (Regex::new(r#"\n[\t\p{Zs}]+"#).unwrap(), "\n"), //newline plus space to newline
            (Regex::new(r#"\n\n+"#).unwrap(), "\n"), //multiple newline to one
        ];

        static ref FINAL_CLEANUP:  [(Regex, &'static str); 2]= [
            (Regex::new(r#"^\s+"#).unwrap(), ""), //remove leading whitespace
            (Regex::new(r#"\s+$"#).unwrap(), ""), //remove end whitespace
        ];
    }

    // TODO, better handle the Cow for performance ?
    let mut res = text.to_owned();

    for (rgx, new_val) in REPLACEMENTS.iter() {
        res = rgx.replace_all(&res, *new_val).to_string();
    }

    // we also dedup the string
    // we dedup all the same 'token' in a line
    // and all the same lines too
    let mut res = res
        .split("\n")
        .map(|s| s.split(", ").dedup().join(", "))
        .dedup()
        .join("\n");

    for (rgx, new_val) in FINAL_CLEANUP.iter() {
        res = rgx.replace(&res, *new_val).to_string();
    }

    let res = res.trim();
    format!("{}\n", res) //add final newline
}

fn replace_before(template: &Template, addr: &mut Address) {
    for r in &template.replace {
        match r {
            ReplaceRule::All(replace_rule) => {
                for c in Component::iter() {
                    if let Some(v) = &addr[c] {
                        addr[c] = Some(
                            replace_rule
                                .regex
                                .replace(&v, replace_rule.replacement_value.as_str())
                                .to_string(),
                        );
                    }
                }
            }
            ReplaceRule::Component((c, replace_rule)) => {
                if let Some(v) = &addr[*c] {
                    addr[*c] = Some(
                        replace_rule
                            .regex
                            .replace(&v, replace_rule.replacement_value.as_str())
                            .to_string(),
                    );
                }
            }
        }
    }
}
