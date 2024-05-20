pub static SUMMARY_TO_JSON_PROMPT:&str = r#"You are an agent dedicated to translating text to JSON. You will receive the text and return it in JSON format.
The format is as follows:


{

    “summary”: “Whole video summary goes here”,
   “talking_points”: [
{
   “title” : “Title of the point”,
   “description: “Talking point summary”
 },
...
]
}

Rules:
- Follow the specified JSON format closely
- Wrap the JSON in a code block
- Skip prose, return only the JSON
"#;

pub fn extract_codeblock(text: &str) -> String {
    if !text.contains("```") {
        return text.to_string();
    }
    let mut in_codeblock = false;
    let mut extracted_lines = vec![];

    for line in text.lines() {
        if line.trim().starts_with("```") {
            in_codeblock = !in_codeblock;
            continue;
        }

        if in_codeblock {
            extracted_lines.push(line);
        }
    }

    extracted_lines.join("\n")
}