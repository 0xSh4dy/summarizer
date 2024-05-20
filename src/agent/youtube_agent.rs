use openai_api_rust::{chat::{ChatApi, ChatBody}, Auth, Error, Message, OpenAI, Role};
use serde::Deserialize;
use youtube_captions::{format::Format, language_tags::LanguageTag, DigestScraper};

use super::json_agent:: SUMMARY_TO_JSON_PROMPT;

static SYSTEM_PROMPT: &str = r#"You are an agent dedicated to summarising video transcripts.
You will receive a transcript and answer with main talking points of the video first,
followed by a complete summary of the transcript. Answer only in this format:


Talking points:
1. ..
2. ..
N. ..

Summary:
Summary of the transcript
"#;

#[derive(Debug)]
pub struct Agent {
    pub system_message: String,
    pub model: String,
    pub client:OpenAI
}

#[derive(Deserialize)]
struct Transcript {
    events: Vec<Event>,
}
#[derive(Deserialize)]
struct Event {
    segs: Option<Vec<Segment>>,
}

#[derive(Deserialize)]
struct Segment {
    utf8: String,
}

async fn get_transcript(video: &str) -> Result<(String), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let digest = DigestScraper::new(client);
    let scraped = digest.fetch(video, "en").await?;
    let language = LanguageTag::parse("en")?;
    let captions = scraped
        .captions
        .into_iter()
        .find(|caption| language.matches(&caption.lang_tag));
    if captions.is_none() {
        return Err("failed to extract captions".to_owned())?;
    }
    let transcript_json = captions.unwrap().fetch(Format::JSON3).await?;
    let root: Transcript = serde_json::from_str(&transcript_json)?;
    let transcript: String = root
        .events
        .iter()
        .filter_map(|event| event.segs.as_ref())
        .flatten()
        .map(|segment| segment.utf8.clone())
        .collect::<Vec<String>>()
        .join(" ");
    Ok(transcript)
}

async fn summarize_video_internal(video: &str) -> Result<String, openai_api_rust::Error> {
    let base_url = "https://api.pawan.krd/pai-001-light/v1/";
    let auth = Auth::from_env().unwrap();
    let client = OpenAI::new(auth, base_url);
    
    let transcript = get_transcript(video).await;

    if transcript.is_err(){
        return Err(openai_api_rust::Error::RequestError(String::from("failed to get transcript")));
    }    
    let mut summarize_agent = Agent {
        system_message: SYSTEM_PROMPT.to_string(),
        model: "gpt-4".to_string(),
        client:client.clone(),
    };
    let mut summary_to_json_agent = Agent {
        system_message: SUMMARY_TO_JSON_PROMPT.to_string(),
        model: "gpt-4".to_string(),
        client:client,
    };

    println!("Running summarizer agent");
    let result = summarize_agent.prompt(transcript.unwrap()).await?;
    Ok(result)
}

pub async fn summarize_video(video:&str)->String{
    let res = summarize_video_internal(video).await ;
    let result = match res{
        Ok(result)=>result,
        Err(err)=>{err.to_string()}
    };
    result
}   

impl Agent {
    pub async fn prompt(&mut self, input: String) -> Result<String,Error> {
        let mut x:String = input.clone().chars().filter(|&c|c!='\n').collect();
        x.truncate(16380);
        let body = ChatBody {
            model: self.get_model(),
            max_tokens: None,
            temperature: Some(0_f32),
            top_p: Some(0_f32),
            n: Some(2),
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: vec![Message {
                role: Role::System,
                content: self.system_message.clone(),
            },Message{
                role:Role::User,
                content:x
            }],
        };
        let rs = self.client.chat_completion_create(&body)?;
        let choices = rs.choices;
        let message = choices[0].message.as_ref();
        if message.is_none(){
            return Ok(String::from("failed to get the output"));
        }
        Ok(message.unwrap().content.clone())
    }
    pub fn get_model(&self) -> String {
        self.model.clone()
    }
}
