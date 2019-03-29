use enum_map::{Enum, EnumMap};
use serde::Serialize;
use strum_macros::{Display, EnumString};

#[derive(Enum, EnumString, Debug, Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Component {
    Attention,
    HouseNumber,
    House,
    Road,
    Village,
    Suburb,
    City,
    County,
    Postcode,
    StateDistrict,
    State,
    Region,
    Island,
    Neighbourhood,
    Country,
    CountryCode,
    Continent,
}

impl serde::Serialize for Component {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Address(EnumMap<Component, Option<String>>);

impl std::ops::Deref for Address {
    type Target = EnumMap<Component, Option<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Address {
    fn deref_mut(&mut self) -> &mut EnumMap<Component, Option<String>> {
        &mut self.0
    }
}
