use colored::Color;

mod helpers;

#[test]
fn hello_world() {
    let client = helpers::Api::new_docker("hello-world", Color::Green);
    client.deploy("../examples/rocket/hello-world");

    let request_text = client
        .request("hello")
        .header("Host", "hello-world-rocket-app.unveil.sh")
        .send()
        .unwrap()
        .text()
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

#[test]
fn postgres() {
    let client = helpers::Api::new_docker("postgres", Color::Blue);
    client.deploy("../examples/rocket/postgres");

    let client = reqwest::blocking::Client::new();
    let add_response = client
        .post("http://localhost:8000/todo")
        .body("{\"note\": \"To the stars\"}")
        .header("Host", "postgres-rocket-app.unveil.sh")
        .send()
        .unwrap()
        .text()
        .unwrap();

    assert_eq!(add_response, "{\"id\":1,\"note\":\"To the stars\"}");

    let fetch_response: String = client
        .get("http://localhost:8000/todo/1")
        .header("Host", "postgres-rocket-app.unveil.sh")
        .send()
        .unwrap()
        .text()
        .unwrap();

    assert_eq!(fetch_response, "{\"id\":1,\"note\":\"To the stars\"}");
}
