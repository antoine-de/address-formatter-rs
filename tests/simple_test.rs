use address_formatter::{Address, Component, Formatter};

#[test]
pub fn basic_test() {
        let formatter = Formatter::default();

        let mut addr = Address::default();
        addr[Component::City] = Some("Toulouse".to_owned());
        addr[Component::Country] = Some("France".to_owned());
        addr[Component::CountryCode] = Some("FR".to_owned());
        addr[Component::County] = Some("Toulouse".to_owned());
        addr[Component::HouseNumber] = Some("17".to_owned());
        addr[Component::Neighbourhood] = Some("Lafourguette".to_owned());
        addr[Component::Postcode] = Some("31000".to_owned());
        addr[Component::Road] = Some("Rue du Médecin-Colonel Calbairac".to_owned());
        addr[Component::State] = Some("Midi-Pyrénées".to_owned());
        addr[Component::Suburb] = Some("Toulouse Ouest".to_owned());

        assert_eq!(
                formatter.format(addr).unwrap(),
                r#"17 Rue du Médecin-Colonel Calbairac
31000 Toulouse
France
"#
                .to_owned()
        )
}
