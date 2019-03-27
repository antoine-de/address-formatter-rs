use address_formatter::{Address, Formatter};
use failure::{format_err, Error};
use include_dir::{include_dir, include_dir_impl};
use yaml_rust::{Yaml, YamlLoader};

#[test]
pub fn opencage_tests() {
    let tests_dir = include_dir!("./address-formatting/testcases/countries");

    let formatter = Formatter::default();
    let errors: Vec<_> = tests_dir
        .files()
        .iter()
        .filter_map(|f| {
            f.contents_utf8().map(|s| {
                (
                    YamlLoader::load_from_str(s).expect(&format!(
                        "impossible to read test file {}",
                        f.path().display()
                    )),
                    f.path().to_str().unwrap(),
                )
            })
        })
        .flat_map(|(s, file_name)| s.into_iter().map(move |s| (s, file_name.clone())))
        .map(|(t, file_name)| run_test(t, &file_name, &formatter))
        .filter_map(|r| r.err())
        .collect();

    if errors.is_empty() {
        println!("All tests ok");
    } else {
        println!("Errors");
        for e in &errors {
            println!("{}", e);
        }

        panic!("{} tests were on error", errors.len());
    }
}

fn run_test(yaml: Yaml, file_name: &str, formatter: &Formatter) -> Result<(), Error> {
    let description = yaml["description"]
        .as_str()
        .unwrap_or("no description provided");
    println!("running {}", &description);

    let expected = yaml["expected"].as_str().ok_or(format_err!(
        "no expected value provided for file {}",
        file_name
    ))?;

    let addr = read_addr(
        yaml["components"]
            .as_hash()
            .ok_or(format_err!("no component value provided {}", file_name))?,
    )?;

    let formated_value = formatter.format(addr)?;

    if formated_value != expected {
        Err(format_err!(
            r#"
====================================
for file {}, test "{}"

expected: 
---
{}
---

got:
----
{}
----
"#,
            file_name,
            description,
            expected,
            formated_value
        ))
    } else {
        Ok(())
    }
}

// unfortunalty, at the time of writing, serde_yaml does not handle multiple documents in a yaml,
// so we have to parse the parse manually
fn read_addr(component: &linked_hash_map::LinkedHashMap<Yaml, Yaml>) -> Result<Address, Error> {
    let get = |component: &linked_hash_map::LinkedHashMap<Yaml, Yaml>, field| {
        component.get(&Yaml::from_str(field)).and_then(|s| {
            s.as_str()
                .map(|s| s.to_string())
                .or_else(|| s.as_i64().map(|s| s.to_string()))
        })
    };
    Ok(Address {
        attention: get(component, "attention"),
        house_number: get(component, "house_number"),
        house: get(component, "house"),
        road: get(component, "road"),
        village: get(component, "village"),
        suburb: get(component, "suburb"),
        city: get(component, "city"),
        county: get(component, "county"),
        postcode: get(component, "postcode"),
        state_district: get(component, "state_district"),
        state: get(component, "state"),
        region: get(component, "region"),
        island: get(component, "island"),
        neighbourhood: get(component, "neighbourhood"),
        country: get(component, "country"),
        country_code: get(component, "country_code"),
        continent: get(component, "continent"),
    })
}
