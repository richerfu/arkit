//! Native map registration, analogous to `echarts.registerMap`.

use std::cell::RefCell;
use std::collections::BTreeMap;

use serde_json::Value;

use crate::model::MapFeature;

thread_local! {
    static MAPS: RefCell<BTreeMap<String, Vec<MapFeature>>> = const { RefCell::new(BTreeMap::new()) };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapRegistrationError {
    pub message: String,
}

impl std::fmt::Display for MapRegistrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MapRegistrationError {}

pub fn register_map(name: impl Into<String>, geo_json: Value) -> Result<(), MapRegistrationError> {
    let features =
        crate::parser::parse_geo_features(&geo_json).ok_or_else(|| MapRegistrationError {
            message: String::from("map must be a GeoJSON FeatureCollection or feature array"),
        })?;
    MAPS.with(|maps| {
        maps.borrow_mut().insert(name.into(), features);
    });
    Ok(())
}

pub fn register_map_str(
    name: impl Into<String>,
    geo_json: &str,
) -> Result<(), MapRegistrationError> {
    let value = serde_json::from_str(geo_json).map_err(|error| MapRegistrationError {
        message: error.to_string(),
    })?;
    register_map(name, value)
}

pub fn unregister_map(name: &str) -> bool {
    MAPS.with(|maps| maps.borrow_mut().remove(name).is_some())
}

pub(crate) fn registered_map(name: &str) -> Option<Vec<MapFeature>> {
    MAPS.with(|maps| maps.borrow().get(name).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_geojson_can_be_resolved_by_name() {
        register_map_str(
            "unit-test-map",
            r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"name":"A"},"geometry":{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,0]]]}}]}"#,
        )
        .unwrap();
        assert_eq!(registered_map("unit-test-map").unwrap()[0].name, "A");
        assert!(unregister_map("unit-test-map"));
    }
}
