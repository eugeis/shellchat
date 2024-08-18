use reqwest::blocking::Client;

use shellchat_lib::InputData;
use shellchat_lib::OutputData;
fn main() {
    let client = Client::new();

    let input_data = InputData { data: "Hello from client".to_string() };

    let res = client
        .post("http://127.0.0.1:8080/execute")
        .json(&input_data)
        .send()
        .unwrap();

    let output_data: OutputData = res.json().unwrap();
    println!("Received: {}", output_data.result);
}
