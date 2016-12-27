extern crate rusoto;
extern crate serde_json;

mod script;
pub mod table;

use std::default::Default;
use std::error::Error;
use std::env::args;

use rusoto::{DefaultCredentialsProvider, Region};
use rusoto::sqs::{SqsClient, ReceiveMessageRequest, DeleteMessageRequest};
use script::write_script;

use serde_json::from_str;
use table::Table;

fn get_table() -> Result<Table, Box<Error>> {
    let queue_url = "https://sqs.us-east-1.amazonaws.com/764188089621/thompson-router".to_string();
    let provider = DefaultCredentialsProvider::new().unwrap();
    let client = SqsClient::new(provider, Region::UsEast1);
    println!("Begin");

    let result = {
        let request = ReceiveMessageRequest {
            max_number_of_messages: Some(1),
            queue_url: queue_url.clone(),
            ..Default::default()
        };

        client.receive_message(&request)?
    };

    let messages = result.messages.ok_or("No messages")?;

    let mut result : Option<Table> = None;
    for message in messages {
        let body = try!(message.body.ok_or("Missing body"));
        let table_result = from_str(&body);
        if let Ok(table) = table_result {
            result = Some(table);
        } else {
            println!("Skipping unparseable message: {:?}", body);
        }

        let handle = message.receipt_handle.ok_or("Missing receipt handle")?;
        let request = DeleteMessageRequest {
            queue_url: queue_url.clone(),
            receipt_handle: handle
        };
        try!(client.delete_message(&request));
    }
    println!("Done");
    let final_result = result.ok_or("No table found")?;
    Ok(final_result)
}

fn main() {
    let args: Vec<String> = args().collect();

    let (new_chain, old_chain) = match args.len() {
        2 => (&args[1], None),
        3 => (&args[1], Some(&args[2])),
        _ => panic!("zzz")
    };

    let table_result = get_table();
    println!("result {:?}", table_result);
    if let Ok(table) = table_result {
        let mut dest = String::new();
        write_script(&table, old_chain, new_chain, &mut dest);
        println!("template\n{}", dest);
    }
}
