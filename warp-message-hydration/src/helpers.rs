use base64::{decode, encode};
use regex::Regex;
use std::collections::VecDeque;

use crate::msg::InputVariable;

pub struct QueueItem {
    pub full_str: String,
    pub base64_str: String,
}

// Helper function to find the base64 substring within a full string
// This function attempts to extract a valid base64 string from the given input string.
pub fn get_base64_string(input_str: String) -> Option<String> {
    // A regular expression that matches base64 strings.
    // This pattern matches groups of four alphanumeric or '+'/'/' characters,
    // followed by either two alphanumeric or '+'/'/' characters and '==',
    // or three alphanumeric or '+'/'/' characters and '='.
    let re = Regex::new(r"(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)").unwrap();

    // Iterate over all captures (matches) of the regular expression in the input string.
    for cap in re.captures_iter(&input_str) {
        // Concatenate "eyJ" and "=" to the captured string to potentially form a valid base64 string.
        let potential_base64 = "eyJ".to_string() + &cap[0] + "=";

        // Attempt to decode the potential base64 string.
        // If the decoding is successful, the string is valid base64 and is returned as the result.
        // If the decoding fails, the function continues with the next capture.
        match decode(potential_base64.clone()) {
            Ok(_) => return Some(potential_base64),
            Err(_) => continue, // Ignore non-base64 strings
        }
    }

    // If no valid base64 string is found, the function returns None.
    None
}

// Helper function to decode a base64 string
// Returns a Result containing either the decoded string or an error
fn decode_base64(base64: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try to decode the base64 string into bytes
    let decoded_bytes = decode(base64)?;
    // Try to convert the bytes into a string
    let decoded_string = String::from_utf8(decoded_bytes)?;
    // Return the decoded string, wrapped in Ok since the function returns a Result
    Ok(decoded_string)
}

// Function that takes a string and a vector of parameters, and returns a Result containing either
// a hydrated string or an error.
pub fn get_hydrated_string(
    input_str: String,
    msg_params: Vec<InputVariable>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Declare a queue of QueueItem structs to keep track of strings and their base64-encoded versions
    let mut queue: VecDeque<QueueItem> = VecDeque::new();

    // Start with the input string
    let mut current_str = input_str;

    // Loop until we break when there's no base64 string to decode
    loop {
        // Try to get the base64-encoded version of the string
        let base64_string = get_base64_string(current_str.clone());

        // If the base64 string exists, decode it, otherwise, break the loop
        if let Some(base64) = base64_string {
            // Try to decode the base64 string and get the original string back
            let decoded_string = decode_base64(&base64)?;
            // Push the original string and its base64 version into the queue
            queue.push_back(QueueItem {
                full_str: current_str,
                base64_str: base64,
            });
            // Update the current string to the decoded string
            current_str = decoded_string;
        } else {
            // If there's no base64 string, push the current string into the queue with an empty base64 version
            // and break the loop
            queue.push_back(QueueItem {
                full_str: current_str,
                base64_str: String::new(),
            });
            break;
        }
    }

    // Start with an empty encoded_hydrated_str
    let mut encoded_hydrated_str = String::new();
    // Start with an empty hydrated_str
    let mut hydrated_str = String::new();

    // While there are items in the queue
    while let Some(queue_item) = queue.pop_back() {
        // Replace the base64 version of the string in the original string with the encoded hydrated string
        let full_str = queue_item
            .full_str
            .replace(&queue_item.base64_str, &encoded_hydrated_str);
        // Update the hydrated string
        hydrated_str = full_str;

        // For each parameter, replace the key in the hydrated string with the value
        for param in &msg_params {
            hydrated_str = hydrated_str.replace(&param.key, &param.value);
        }

        // Re-encode the hydrated string
        encoded_hydrated_str = encode(hydrated_str.clone());
    }

    // Return the hydrated string, wrapped in Ok since the function returns a Result
    Ok(hydrated_str)
}
