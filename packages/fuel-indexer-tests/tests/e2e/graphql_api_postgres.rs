use fuel_indexer_tests::fixtures::{
    http_client, mock_request, setup_web_test_components, WebTestComponents,
};
use hyper::header::CONTENT_TYPE;
use serde_json::{Number, Value};
use std::collections::HashMap;

#[actix_web::test]
async fn test_can_return_query_response_with_all_fields_required_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { blockentity { id height timestamp }}" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["height"].as_u64().unwrap(), 0);
    assert_eq!(data[0]["timestamp"].as_u64().unwrap(), 0);

    assert!(data[1]["height"].as_u64().unwrap() > 0);
    assert!(data[1]["timestamp"].as_u64().unwrap() > 0);

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_nullable_fields_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/optionals").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { optionentity { int_required int_optional_some addr_optional_none }}"}"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["int_required"], Value::from(Number::from(100)));
    assert_eq!(data[0]["int_optional_some"], Value::from(Number::from(999)));
    assert_eq!(data[0]["addr_optional_none"], Value::from(None::<&str>));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_nested_query_response_with_implicit_foreign_keys_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/block").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { txentity { block { id height } id timestamp } }" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_i64().is_some());
    assert!(data[0]["id"].as_i64().unwrap() > 0);
    assert!(data[0]["timestamp"].as_i64().is_some());
    assert!(data[0]["timestamp"].as_i64().unwrap() > 0);
    assert!(data[0]["block"]["id"].as_i64().is_some());
    assert!(data[0]["block"]["id"].as_i64().unwrap() > 0);
    assert!(data[0]["block"]["height"].as_i64().is_some());
    assert!(data[0]["block"]["height"].as_i64().unwrap() > 0);

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_deeply_nested_query_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/deeply_nested").await;

    let deeply_nested_query = HashMap::from([(
        "query",
        "query {
                bookclub {
                    id
                    book {
                        id
                        name
                        author {
                            name
                            genre {
                                id
                                name
                            }
                        }
                        library {
                            id
                            name
                            city {
                                id
                                name
                                region {
                                    id
                                    name
                                    country {
                                        id
                                        name
                                        continent {
                                            id
                                            name
                                            planet {
                                                id
                                                name
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        genre {
                            id
                            name
                        }
                    }
                    member {
                        name
                        id
                    }
                    corporate_sponsor {
                        id
                        name
                        amount
                        representative {
                            id
                            name
                        }
                    }
                }
            }",
    )]);

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .json(&deeply_nested_query)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    // Multiple reference to same foreign key table
    assert_eq!(
        data[0]["book"]["author"]["genre"]["name"].as_str(),
        Some("horror")
    );
    assert_eq!(data[0]["book"]["genre"]["name"].as_str(), Some("horror"));

    // Deeply nested foreign keys
    assert_eq!(
        data[0]["book"]["library"]["name"].as_str(),
        Some("Scholar Library")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["name"].as_str(),
        Some("Savanna-la-Mar")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["name"].as_str(),
        Some("Westmoreland")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["country"]["name"].as_str(),
        Some("Jamaica")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["country"]["continent"]["name"]
            .as_str(),
        Some("North America")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["country"]["continent"]["planet"]
            ["name"]
            .as_str(),
        Some("Earth")
    );

    // Mix of implicit and explicit foreign keys as well as
    // field name being different from underlying database table
    assert_eq!(
        data[0]["corporate_sponsor"]["name"].as_str(),
        Some("Fuel Labs")
    );
    assert_eq!(data[0]["corporate_sponsor"]["amount"].as_i64(), Some(100));
    assert_eq!(
        data[0]["corporate_sponsor"]["representative"]["name"].as_str(),
        Some("Ava")
    );

    server.abort();
}

#[actix_web::test]
async fn test_can_return_nested_query_response_with_explicit_foreign_keys_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/explicit").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { sportsteam { id name municipality { id name } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["name"].as_str(), Some("The Indexers"));
    assert!(data[0]["municipality"]["id"].as_i64().is_some());
    assert!(data[0]["municipality"]["id"].as_i64().unwrap() > 0);
    assert_eq!(
        data[0]["municipality"]["name"].as_str(),
        Some("Republic of Indexia")
    );

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_id_selection_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { filterentity(id: 1) { id foola maybe_null_bar bazoo } }" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_membership_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { foola: { in: [\"beep\", \"boop\"] } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(2));
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_non_null_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar] } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(3));
    assert_eq!(data[1]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[1]["bazoo"].as_i64(), Some(1000));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_complex_comparison_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { bazoo: { between: { min: 0, max: 10 } } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(2));
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_simple_comparison_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;
    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { filterentity(filter: { bazoo: { lt: 1000 } } ) { id foola maybe_null_bar bazoo } }" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(2));
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_nested_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar] } ) { id foola maybe_null_bar bazoo inner_entity(filter: { inner_foo: { in: [\"ham\", \"eggs\"] } } ) { id inner_foo inner_bar inner_baz } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(3));
    assert_eq!(data[0]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1000));
    assert_eq!(data[0]["inner_entity"]["id"].as_i64(), Some(3));
    assert_eq!(data[0]["inner_entity"]["inner_foo"].as_str(), Some("eggs"));
    assert_eq!(data[0]["inner_entity"]["inner_bar"].as_u64(), Some(500));
    assert_eq!(data[0]["inner_entity"]["inner_baz"].as_u64(), Some(600));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_multiple_on_single_entity_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar], and: { bazoo: { equals: 1 } } } ) { id foola maybe_null_bar bazoo inner_entity { id inner_foo inner_bar inner_baz } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_filter_negation_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { filterentity(filter: { not: { foola: { in: [\"beep\", \"boop\"] } } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(3));
    assert_eq!(data[0]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1000));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_sorted_results_postgres() {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;
    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { filterentity(order: { foola: desc }) { id foola } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(2));
    assert_eq!(data[0]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["id"].as_i64(), Some(3));
    assert_eq!(data[1]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[2]["id"].as_i64(), Some(1));
    assert_eq!(data[2]["foola"].as_str(), Some("beep"));

    server.abort();
}

#[actix_web::test]
async fn test_can_return_query_response_with_alias_and_ascending_offset_and_limited_results_postgres(
) {
    let WebTestComponents {
        server, db: _db, ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { aliased_entities: filterentity(order: { foola: asc }, first: 1, offset: 1) { id foola } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["aliased_entities"][0]["id"].as_i64(), Some(3));
    assert_eq!(
        data[0]["aliased_entities"][0]["foola"].as_str(),
        Some("blorp")
    );
    assert_eq!(data[0]["page_info"]["pages"].as_i64(), Some(3));

    server.abort();
}
