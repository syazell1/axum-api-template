use claims::assert_ok;
use serde_json::json;
use test_rs::{
    features::auth::controller::AuthResponse, utils::randomizer::generate_random_string,
};

use crate::helpers::{spawn_app, TestUser};

#[tokio::test]
pub async fn a_valid_credentials_is_accepted() {
    // arrange
    let app = spawn_app().await;
    app.test_user.store_user(&app.pool).await;

    // act
    let res = app.login_user(&json!(app.test_user)).await;
    let refresh_token = res.headers().get("set-cookie").is_some();

    // assert
    assert_eq!(200, res.status().as_u16());
    assert_ok!(res.json::<AuthResponse>().await);
    assert_eq!(true, refresh_token);
}

#[tokio::test]
pub async fn an_invalid_username_is_rejected_and_would_return_401() {
    // arrange
    let app = spawn_app().await;
    let mut test_user = TestUser::generate();
    test_user.store_user(&app.pool).await;

    test_user.username = generate_random_string(12);

    // act
    let res = app.login_user(&json!(test_user)).await;

    // assert
    assert_eq!(401, res.status().as_u16());
}

#[tokio::test]
pub async fn an_invalid_password_is_rejected_and_would_return_401() {
    // arrange
    let app = spawn_app().await;
    let mut test_user = TestUser::generate();
    test_user.store_user(&app.pool).await;

    test_user.password = generate_random_string(12);

    // act
    let res = app.login_user(&json!(test_user)).await;

    // assert
    assert_eq!(401, res.status().as_u16());
}

#[tokio::test]
pub async fn missing_field_would_return_422() {
    // Arrange
    let app = spawn_app().await;
    let credentials = vec![
        json!({"username": generate_random_string(12)}),
        json!({"password": generate_random_string(12)}),
        json!({}),
    ];

    for c in &credentials {
        // Act
        let res = app.login_user(&c).await;

        // Assert
        assert_eq!(422, res.status().as_u16());
    }
}

#[tokio::test]
pub async fn login_returns_400_when_credentials_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let credentials = vec![
        json!({"username": generate_random_string(24), "password": generate_random_string(12)}),
        json!({"username": generate_random_string(12), "password": generate_random_string(24)}),
        json!({"username": "", "password": generate_random_string(12)}),
        json!({"username": generate_random_string(12), "password": ""}),
    ];

    for c in &credentials {
        // Act
        let res = app.login_user(c).await;

        // Assert
        assert_eq!(400, res.status().as_u16());
    }
}
