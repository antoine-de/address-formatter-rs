use crate::{Address, Component};
use failure::Fail;
use failure::{format_err, Error};
use itertools::Itertools;
use regex::Regex;
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
    pub postformat_replace: Vec<Replacement>,
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
    pub(crate) component_aliases: HashMap<Component, Vec<String>>,
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
        let text = cleanup_rendered(&text, &template);

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
        let mut unknown = HashMap::<String, String>::new();
        for (k, v) in values.into_iter() {
            let component = Component::from_str(k).ok();;
            if let Some(component) = component {
                address[component] = Some(v);
            } else {
                unknown.insert(k.to_string(), v);
            }
        }
        // all the unknown fields are added in the 'Attention' field
        if !unknown.is_empty() {
            for (c, aliases) in &self.component_aliases {
                // if the address's component has not been already set, we set it to it's first found alias
                for alias in aliases {
                    if let Some(a) = unknown.remove(alias) {
                        if address[*c].is_none() {
                            address[*c] = Some(a);
                            break;
                        }
                    }
                }
            }
            address[Component::Attention] = Some(unknown.values().into_iter().join(", "));
        }
        address
    }
}

fn sanity_clean_address(addr: &mut Address) {
    lazy_static::lazy_static! {
        static ref POST_CODE_RANGE:  Regex= Regex::new(r#"\d+;\d+"#).unwrap();
        static ref MATCHABLE_POST_CODE_RANGE:  Regex= Regex::new(r#"^(\d{5}),\d{5}"#).unwrap();
        static ref IS_URL:  Regex= Regex::new(r#"https?://"#).unwrap();

    }
    // cleanup the postcode
    if let Some(post_code) = &addr[Component::Postcode] {
        if post_code.len() > 20 {
            addr[Component::Postcode] = None;
        } else if POST_CODE_RANGE.is_match(post_code) {
            addr[Component::Postcode] = None;
        } else if let Some(r) = MATCHABLE_POST_CODE_RANGE
            .captures(post_code)
            .and_then(|r| r.get(1))
            .map(|c| c.as_str())
        {
            addr[Component::Postcode] = Some(r.to_owned());
        }
    }

    // clean values containing URLs
    for c in Component::iter() {
        if let Some(v) = &addr[c] {
            if IS_URL.is_match(v) {
                addr[Component::Postcode] = None;
            }
        }
    }
    /*

    if let (Some(country),Some( state)) = (addr[Component::Country], addr[Component::State]) {

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

fn cleanup_rendered(text: &str, template: &Template) -> String {
    use itertools::Itertools;
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

    for r in &template.postformat_replace {
        res = r
            .regex
            .replace(&res, r.replacement_value.as_str())
            .to_string();
    }

    let res = res.trim();
    format!("{}\n", res) //add final newline
}

fn has_minimum_address_components(_addr: &Address) -> bool {
    //TODO
    true
}

fn replace_before(template: &Template, addr: &mut Address) {
    for r in &template.replace {
        r.replace_fields(addr);
    }
}

impl ReplaceRule {
    fn replace_fields<'a>(&self, addr: &mut Address) {
        match self {
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
