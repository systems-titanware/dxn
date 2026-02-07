use super::*;
use serde_json::json;
use uuid::Uuid;

/// Generate a unique test ID to avoid conflicts between parallel tests
fn unique_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::now_v7())
}

#[test]
fn test_init_events_table() {
    // Initialize table (idempotent - uses CREATE TABLE IF NOT EXISTS)
    let result = init_events_table();
    assert!(result.is_ok(), "Should create events table");
    
    // Calling again should not fail (idempotent)
    let result2 = init_events_table();
    assert!(result2.is_ok(), "Should be idempotent");
}

#[test]
fn test_append_and_get_event() {
    init_events_table().unwrap();
    
    let event_id = unique_id("test-event");
    let aggregate_id = unique_id("order");
    
    let event = Event {
        id: event_id.clone(),
        aggregate_id: aggregate_id.clone(),
        schema_name: "order".to_string(),
        event_type: EventType::Created,
        payload: json!({"item": "Widget", "quantity": 5}),
        previous_state: None,
        version: 1,
        user_id: Some("user-1".to_string()),
        timestamp: "2024-01-15T10:00:00Z".to_string(),
    };
    
    let result = append_event(&event);
    assert!(result.is_ok(), "Should append event");
    
    let retrieved = get_event_by_id(&event_id);
    assert!(retrieved.is_ok(), "Should retrieve event");
    
    let retrieved_event = retrieved.unwrap();
    assert_eq!(retrieved_event.aggregate_id, aggregate_id);
    assert_eq!(retrieved_event.schema_name, "order");
    assert_eq!(retrieved_event.event_type, EventType::Created);
}

#[test]
fn test_get_events_by_aggregate() {
    init_events_table().unwrap();
    
    let aggregate_id = unique_id("order");
    let event_id_1 = unique_id("evt");
    let event_id_2 = unique_id("evt");
    
    // Create multiple events for the same aggregate
    let events = vec![
        Event {
            id: event_id_1,
            aggregate_id: aggregate_id.clone(),
            schema_name: "order".to_string(),
            event_type: EventType::Created,
            payload: json!({"status": "pending"}),
            previous_state: None,
            version: 1,
            user_id: None,
            timestamp: "2024-01-15T10:00:00Z".to_string(),
        },
        Event {
            id: event_id_2,
            aggregate_id: aggregate_id.clone(),
            schema_name: "order".to_string(),
            event_type: EventType::Updated,
            payload: json!({"status": "processing"}),
            previous_state: Some(json!({"status": "pending"})),
            version: 2,
            user_id: None,
            timestamp: "2024-01-15T11:00:00Z".to_string(),
        },
    ];
    
    for event in &events {
        append_event(event).unwrap();
    }
    
    let result = get_events_by_aggregate(&aggregate_id);
    assert!(result.is_ok());
    
    let retrieved = result.unwrap();
    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved[0].version, 1);
    assert_eq!(retrieved[1].version, 2);
}

#[test]
fn test_create_and_append_event() {
    init_events_table().unwrap();
    
    let aggregate_id = unique_id("profile");
    
    let result = create_and_append_event(
        &aggregate_id,
        "profile",
        EventType::Created,
        json!({"name": "John Doe", "email": "john@example.com"}),
        None,
        Some("admin".to_string()),
    );
    
    assert!(result.is_ok());
    let event = result.unwrap();
    
    assert_eq!(event.aggregate_id, aggregate_id);
    assert_eq!(event.schema_name, "profile");
    assert_eq!(event.version, 1);
    assert!(event.user_id.is_some());
}

#[test]
fn test_replay_aggregate() {
    init_events_table().unwrap();
    
    let aggregate_id = unique_id("product");
    
    // Simulate a lifecycle: create -> update -> update
    let _ = create_and_append_event(
        &aggregate_id,
        "product",
        EventType::Created,
        json!({"name": "Widget", "price": 10.0}),
        None,
        None,
    );
    
    let _ = create_and_append_event(
        &aggregate_id,
        "product",
        EventType::Updated,
        json!({"price": 12.0}),
        None,
        None,
    );
    
    let _ = create_and_append_event(
        &aggregate_id,
        "product",
        EventType::Updated,
        json!({"stock": 100}),
        None,
        None,
    );
    
    let result = replay_aggregate(&aggregate_id);
    assert!(result.is_ok());
    
    let state = result.unwrap();
    assert!(state.is_some());
    
    let state_obj = state.unwrap();
    assert_eq!(state_obj["name"], "Widget");
    assert_eq!(state_obj["price"], 12.0);
    assert_eq!(state_obj["stock"], 100);
    assert_eq!(state_obj["_version"], 3);
}

#[test]
fn test_event_type_conversion() {
    assert_eq!(EventType::Created.as_str(), "created");
    assert_eq!(EventType::Updated.as_str(), "updated");
    assert_eq!(EventType::Deleted.as_str(), "deleted");
    assert_eq!(EventType::Custom("shipped".to_string()).as_str(), "shipped");
    
    assert_eq!(EventType::from_str("created"), EventType::Created);
    assert_eq!(EventType::from_str("UPDATED"), EventType::Updated);
    assert_eq!(EventType::from_str("shipped"), EventType::Custom("shipped".to_string()));
}

#[test]
fn test_version_increment() {
    init_events_table().unwrap();
    
    let aggregate_id = unique_id("new-aggregate");
    
    // First event should be version 1
    let v1 = get_next_version(&aggregate_id);
    assert!(v1.is_ok());
    assert_eq!(v1.unwrap(), 1);
    
    // Create an event
    let _ = create_and_append_event(
        &aggregate_id,
        "test",
        EventType::Created,
        json!({}),
        None,
        None,
    );
    
    // Next should be version 2
    let v2 = get_next_version(&aggregate_id);
    assert!(v2.is_ok());
    assert_eq!(v2.unwrap(), 2);
}
