use crate::formatter::{
    CountryCode, Formatter, NewComponent, ReplaceRule, Replacement, Template, Templates,
};
use crate::Component;
use std::collections::HashMap;
use std::str::FromStr;

pub fn read_configuration() -> Formatter {
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

    let mut component_aliases = HashMap::<_, _>::new();
    for c in &raw_components {
        if let Some(aliases) = c["aliases"].as_vec() {
            for a in aliases {
                let name = c["name"].as_str().unwrap();
                component_aliases.insert(
                    a.as_str().unwrap().to_string(),
                    Component::from_str(name).expect(&format!("{} is not a valid component", name)),
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
            if let Some(parent_country) = v["use_country"]
                .as_str()
                .and_then(|k| CountryCode::from_str(k).ok())
            {
                // we store it for later processing
                overrided_countries.insert(country_code, (parent_country, v.clone()));
                None
            } else {
                let replace_rules = v["replace"]
                    .as_vec()
                    .map(|v| {
                        v.iter()
                            .map(|r| {
                                let r = r.as_vec().expect("replace should be a list");
                                //TODO handle component=regex
                                assert_eq!(r.len(), 2);

                                let first_val = r[0].as_str().expect("invalid replace rule");
                                if first_val.contains("=") {
                                    // it's a replace on only one component
                                    // the rules is written 'component=<string_to_replace'
                                    let parts =
                                        first_val.split("=").into_iter().collect::<Vec<_>>();
                                    let component = Component::from_str(parts[0]).expect(&format!(
                                        "in replace '{}' is not a valid component",
                                        parts[0]
                                    ));
                                    ReplaceRule::Component((
                                        component,
                                        Replacement {
                                            regex: regex::Regex::new(parts[1])
                                                .expect("invalid regex"),
                                            replacement_value: r[1]
                                                .as_str()
                                                .expect("invalid replace rule")
                                                .to_owned(),
                                        },
                                    ))
                                } else {
                                    // it's a replace for all components
                                    ReplaceRule::All(Replacement {
                                        regex: regex::Regex::new(first_val).expect("invalid regex"),
                                        replacement_value: r[1]
                                            .as_str()
                                            .expect("invalid replace rule")
                                            .to_owned(),
                                    })
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_else(|| vec![]);
                let template = Template {
                    address_template: v["address_template"]
                        .as_str()
                        .expect(&format!(
                            "no address_template found for country {}",
                            country_code
                        ))
                        .to_string(),
                    replace: replace_rules,
                    //TODO replace & postformat
                    ..Default::default()
                };
                Some((country_code, template))
            }
        })
        .collect();

    for (country_code, (parent_country_code, template)) in overrided_countries.into_iter() {
        let mut overrided_template = templates_by_country[&parent_country_code].clone();

        overrided_template.change_country =
            template["change_country"].as_str().map(|s| s.to_string());
        if let Some(add_component) = template["add_component"].as_str() {
            let part: Vec<_> = add_component.split("=").collect();
            assert_eq!(part.len(), 2);
            overrided_template.add_component = Some(NewComponent {
                component: part[0].to_owned(),
                new_value: part[1].to_owned(),
            })
        }
        templates_by_country.insert(country_code, overrided_template);
    }

    let templates = Templates {
        default_template,
        fallback_template,
        templates_by_country,
    };
    Formatter {
        components,
        component_aliases,
        templates,
    }
}
