use address_formatter::{Address, Formatter};

#[test]
pub fn basic_test() {
    let formatter = Formatter::default();

    let addr = Address {
        city: Some("Toulouse".to_owned()),
        country: Some("France".to_owned()),
        country_code: Some("FR".to_owned()),
        county: Some("Toulouse".to_owned()),
        house_number: Some("17".to_owned()),
        neighbourhood: Some("Lafourguette".to_owned()),
        postcode: Some("31000".to_owned()),
        road: Some("Rue du Médecin-Colonel Calbairac".to_owned()),
        state: Some("Midi-Pyrénées".to_owned()),
        suburb: Some("Toulouse Ouest".to_owned()),
        ..Default::default()
    };

    assert_eq!(
        formatter.format(addr).unwrap(),
        r#"
17 Rue du Médecin-Colonel Calbairac
31000 Toulouse
France"#
            .to_owned()
    )
}

#[test]
pub fn yaml_test() {
    use yaml_rust::{YamlEmitter, YamlLoader};
    let s = "
name: house_number
aliases:
    - street_number
---
name: house
aliases:
    - building
";
    let docs = YamlLoader::load_from_str(s).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs;

    // Debug support
    println!("{:?}", doc);
}
