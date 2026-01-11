use super::*;
use crate::system::server::models::SystemServerRoute;

#[test]
fn test_flatten_routes_empty() {
    let result = flatten_routes(None);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_flatten_routes_single_route() {
    let routes = Some(vec![SystemServerRoute {
        name: "home".to_string(),
        file: "home.html".to_string(),
        function: None,
        routes: None,
    }]);

    let result = flatten_routes(routes);
    
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("home"));
    let route = result.get("home").unwrap();
    assert_eq!(route.name, "home");
    assert_eq!(route.file, "home.html");
    assert_eq!(route.function, None);
}

#[test]
fn test_flatten_routes_nested() {
    let routes = Some(vec![SystemServerRoute {
        name: "blog".to_string(),
        file: "blog.html".to_string(),
        function: None,
        routes: Some(vec![
            SystemServerRoute {
                name: "post".to_string(),
                file: "post.html".to_string(),
                function: None,
                routes: None,
            },
            SystemServerRoute {
                name: "archive".to_string(),
                file: "archive.html".to_string(),
                function: None,
                routes: None,
            },
        ]),
    }]);

    let result = flatten_routes(routes);
    
    assert_eq!(result.len(), 3);
    assert!(result.contains_key("blog"));
    assert!(result.contains_key("blog/post"));
    assert!(result.contains_key("blog/archive"));
    
    let blog_route = result.get("blog").unwrap();
    assert_eq!(blog_route.name, "blog");
    
    let post_route = result.get("blog/post").unwrap();
    assert_eq!(post_route.name, "post");
    
    let archive_route = result.get("blog/archive").unwrap();
    assert_eq!(archive_route.name, "archive");
}

#[test]
fn test_flatten_routes_deeply_nested() {
    let routes = Some(vec![SystemServerRoute {
        name: "level1".to_string(),
        file: "level1.html".to_string(),
        function: None,
        routes: Some(vec![SystemServerRoute {
            name: "level2".to_string(),
            file: "level2.html".to_string(),
            function: None,
            routes: Some(vec![SystemServerRoute {
                name: "level3".to_string(),
                file: "level3.html".to_string(),
                function: None,
                routes: None,
            }]),
        }]),
    }]);

    let result = flatten_routes(routes);
    
    assert_eq!(result.len(), 3);
    assert!(result.contains_key("level1"));
    assert!(result.contains_key("level1/level2"));
    assert!(result.contains_key("level1/level2/level3"));
}

#[test]
fn test_flatten_routes_with_function() {
    let routes = Some(vec![SystemServerRoute {
        name: "test".to_string(),
        file: "test.html".to_string(),
        function: Some("parse_markdown".to_string()),
        routes: None,
    }]);

    let result = flatten_routes(routes);
    
    assert_eq!(result.len(), 1);
    let route = result.get("test").unwrap();
    assert_eq!(route.function, Some("parse_markdown".to_string()));
}

#[test]
fn test_recursively_flatten_routes_single() {
    let mut map = HashMap::new();
    let route = SystemServerRoute {
        name: "single".to_string(),
        file: "single.html".to_string(),
        function: None,
        routes: None,
    };

    recursively_flatten_routes(route, &mut map, "");
    
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("single"));
}

#[test]
fn test_recursively_flatten_routes_with_parent() {
    let mut map = HashMap::new();
    let route = SystemServerRoute {
        name: "child".to_string(),
        file: "child.html".to_string(),
        function: None,
        routes: None,
    };

    recursively_flatten_routes(route, &mut map, "parent");
    
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("parent/child"));
    assert!(!map.contains_key("child"));
}

#[test]
fn test_recursively_flatten_routes_multiple_children() {
    let mut map = HashMap::new();
    let route = SystemServerRoute {
        name: "parent".to_string(),
        file: "parent.html".to_string(),
        function: None,
        routes: Some(vec![
            SystemServerRoute {
                name: "child1".to_string(),
                file: "child1.html".to_string(),
                function: None,
                routes: None,
            },
            SystemServerRoute {
                name: "child2".to_string(),
                file: "child2.html".to_string(),
                function: None,
                routes: None,
            },
        ]),
    };

    recursively_flatten_routes(route, &mut map, "");
    
    assert_eq!(map.len(), 3);
    assert!(map.contains_key("parent"));
    assert!(map.contains_key("parent/child1"));
    assert!(map.contains_key("parent/child2"));
}

#[test]
fn test_convert_routes() {
    let mut routes = HashMap::new();
    routes.insert("test".to_string(), FlattenRoutePath {
        name: "test".to_string(),
        file: "test.html".to_string(),
        function: None,
        params: None,
    });
    routes.insert("blog".to_string(), FlattenRoutePath {
        name: "blog".to_string(),
        file: "blog.html".to_string(),
        function: Some("parse".to_string()),
        params: None,
    });

    let result = convert_routes(routes);
    
    // Verify that resources were created (we can't easily test the actix Resource type,
    // but we can verify the count)
    assert_eq!(result.len(), 2);
}

#[test]
fn test_convert_routes_empty() {
    let routes = HashMap::new();
    let result = convert_routes(routes);
    assert_eq!(result.len(), 0);
}

