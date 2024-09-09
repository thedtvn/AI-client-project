use rand::{thread_rng, Rng};

use crate::serde_obj::{TokenResponse, ToolCallFn};

pub fn generate_random_string(len: usize) -> String {
    let mut rng = thread_rng();
    (0..len)
        .map(|_| {
            let c = rng.gen_range(0..36); // Generate a number between 0 and 35
            if c < 10 {
                (b'0' + c as u8) as char // Convert to '0'-'9'
            } else {
                (b'a' + (c - 10) as u8) as char // Convert to 'a'-'z'
            }
        })
        .collect()
}

pub fn get_response_token(data: String) -> TokenResponse {
    let vec_data: Vec<String> = serde_json::from_str(&data).unwrap();
    let json_str_data = vec_data[0].clone();
    serde_json::from_str(&json_str_data).unwrap()
}


pub fn prase_tool_call(input: String) -> Result<Vec<ToolCallFn>, serde_json::Error> {
    let input = input.split("\n\n").collect::<Vec<&str>>();
    let tool_call: Vec<ToolCallFn> = serde_json::from_str(&input[0])?;
    let mut new_tool_call_with_id = Vec::new();
    for mut tool in tool_call {
        let call_id = generate_random_string(9);
        tool.call_id = Some(call_id);
        new_tool_call_with_id.push(tool);
    }
    Ok(new_tool_call_with_id)
}