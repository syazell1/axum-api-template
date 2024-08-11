use crate::helpers::spawn_app;

#[tokio::test]
pub async fn health_check_should_return_success_200() {
    // arrange
    let app = spawn_app().await;

    // act
    let res = app.http_client.get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to send request.");

    // assert
    assert_eq!(res.status().as_u16(), 200);
}