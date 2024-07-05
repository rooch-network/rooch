// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::RegistryService;
use prometheus::{IntCounter, Registry};

#[test]
fn registry_service() {
    // GIVEN
    let default_registry = Registry::new_custom(Some("default".to_string()), None).unwrap();

    let registry_service = RegistryService::new(default_registry.clone());
    let default_counter = IntCounter::new("counter", "counter_desc").unwrap();
    default_counter.inc();
    default_registry
        .register(Box::new(default_counter))
        .unwrap();

    // AND add a metric to the default registry

    // AND a registry with one metric
    let registry_1 = Registry::new_custom(Some("narwhal".to_string()), None).unwrap();
    registry_1
        .register(Box::new(
            IntCounter::new("counter_1", "counter_1_desc").unwrap(),
        ))
        .unwrap();

    // WHEN
    let registry_1_id = registry_service.add(registry_1);

    // THEN
    let mut metrics = registry_service.gather_all();
    metrics.sort_by(|m1, m2| Ord::cmp(m1.get_name(), m2.get_name()));

    assert_eq!(metrics.len(), 2);

    let metric_default = metrics.remove(0);
    assert_eq!(metric_default.get_name(), "default_counter");
    assert_eq!(metric_default.get_help(), "counter_desc");

    let metric_1 = metrics.remove(0);
    assert_eq!(metric_1.get_name(), "narwhal_counter_1");
    assert_eq!(metric_1.get_help(), "counter_1_desc");

    // AND add a second registry with a metric
    let registry_2 = Registry::new_custom(Some("rooch".to_string()), None).unwrap();
    registry_2
        .register(Box::new(
            IntCounter::new("counter_2", "counter_2_desc").unwrap(),
        ))
        .unwrap();
    let _registry_2_id = registry_service.add(registry_2);

    // THEN all the metrics should be returned
    let mut metrics = registry_service.gather_all();
    metrics.sort_by(|m1, m2| Ord::cmp(m1.get_name(), m2.get_name()));

    assert_eq!(metrics.len(), 3);

    let metric_default = metrics.remove(0);
    assert_eq!(metric_default.get_name(), "default_counter");
    assert_eq!(metric_default.get_help(), "counter_desc");

    let metric_1 = metrics.remove(0);
    assert_eq!(metric_1.get_name(), "narwhal_counter_1");
    assert_eq!(metric_1.get_help(), "counter_1_desc");

    let metric_2 = metrics.remove(0);
    assert_eq!(metric_2.get_name(), "rooch_counter_2");
    assert_eq!(metric_2.get_help(), "counter_2_desc");

    // AND remove first registry
    assert!(registry_service.remove(registry_1_id));

    // THEN metrics should now not contain metric of registry_1
    let mut metrics = registry_service.gather_all();
    metrics.sort_by(|m1, m2| Ord::cmp(m1.get_name(), m2.get_name()));

    assert_eq!(metrics.len(), 2);

    let metric_default = metrics.remove(0);
    assert_eq!(metric_default.get_name(), "default_counter");
    assert_eq!(metric_default.get_help(), "counter_desc");

    let metric_1 = metrics.remove(0);
    assert_eq!(metric_1.get_name(), "rooch_counter_2");
    assert_eq!(metric_1.get_help(), "counter_2_desc");
}
