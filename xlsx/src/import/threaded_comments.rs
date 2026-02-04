use std::io::Read;

use chrono::NaiveDateTime;
use ironcalc_base::types::{Mention, Person, Provider, ThreadedComment};
use roxmltree::Node;
use uuid::Uuid;

use crate::error::XlsxError;

// pub(crate) fn read_threaded_comments_and_people<R: Read + std::io::Seek>(
//     archive: &mut zip::read::ZipArchive<R>,
// ) -> Result<(Vec<Person>, Vec<ThreadedComment>), XlsxError> {
//     let person_xml = "xl/persons/person.xml";
//     let people = match archive.by_name(person_xml) {
//         Ok(mut file) => {
//             let mut text = String::new();
//             file.read_to_string(&mut text)?;
//             read_people_from_string(&text)?
//         }
//         Err(_e) => Vec::new(),
//     };

//     let threaded_prefix = "xl/threadedComments";

//     let mut comments: Vec<ThreadedComment> = Vec::new();
//     let mut references:
//     for i in 0..archive.len() {
//         let mut file = archive.by_index(i)?;
//         if !file.name().starts_with(threaded_prefix) {
//             continue;
//         }
//         let mut text = String::new();
//         file.read_to_string(&mut text)?;
//         let sheet_comments = read_comments_from_string(&text)?;
//         comments.extend(sheet_comments);
//     }
//     Ok((people, comments))
// }

pub fn read_persons_from_archive<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<Vec<Person>, XlsxError> {
    let person_xml = "xl/persons/person.xml";
    match archive.by_name(person_xml) {
        Ok(mut file) => {
            let mut text = String::new();
            file.read_to_string(&mut text)?;
            read_people_from_string(&text)
        }
        Err(_e) => Ok(vec![]),
    }
}
fn read_people_from_string(text: &str) -> Result<Vec<Person>, XlsxError> {
    let doc = roxmltree::Document::parse(text)?;
    let person_list_node = doc.descendants().find(|n| n.has_tag_name("personList"));
    match person_list_node {
        Some(person_list_node) => parse_people_from_person_list_node(person_list_node),
        None => Ok(Vec::new()),
    }
}
fn str_to_uuid(raw_str: &str) -> Result<Uuid, XlsxError> {
    Uuid::parse_str(raw_str.trim_matches(|c| c == '{' || c == '}'))
        .map_err(|_| XlsxError::Xml("Can't get person UUID".to_string()))
}
fn parse_people_from_person_list_node(person_list_node: Node) -> Result<Vec<Person>, XlsxError> {
    let mut persons: Vec<Person> = Vec::new();
    for person_node in person_list_node
        .children()
        .filter(|n| n.has_tag_name("person"))
    {
        let display_name = person_node.attribute("displayName").unwrap_or_default();
        let raw_id = person_node.attribute("id").unwrap_or_default();
        let person_id = str_to_uuid(raw_id)?;

        let user_id = person_node.attribute("userId");

        let provider_str = person_node.attribute("provider").unwrap_or("Other");
        let provider = Provider::from_str_and_user_id(provider_str, user_id)
            .map_err(|e| XlsxError::Xml(e.to_string()))?;
        let mut ext_lst = Vec::new();
        if let Some(ext_lst_node) = person_node.children().find(|n| n.has_tag_name("extLst")) {
            for ext in ext_lst_node.children().filter(|n| n.is_element()) {
                if let Some(text) = ext.text() {
                    ext_lst.push(text.to_string());
                }
            }
        }
        persons.push(Person::new(
            ext_lst,
            display_name,
            person_id,
            user_id,
            provider,
        ))
    }
    Ok(persons)
}

pub fn read_threaded_from_archive<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
) -> Result<Vec<ThreadedComment>, XlsxError> {
    match archive.by_name(path) {
        Ok(mut file) => {
            let mut text = String::new();
            file.read_to_string(&mut text)?;
            read_comments_from_string(&text)
        }
        Err(_e) => Ok(vec![]),
    }
}
fn read_comments_from_string(text: &str) -> Result<Vec<ThreadedComment>, XlsxError> {
    let doc = roxmltree::Document::parse(text)?;
    let comment_list_node = doc
        .descendants()
        .find(|n| n.has_tag_name("ThreadedComments"));
    match comment_list_node {
        Some(comment_list_node) => parse_comments_from_comment_list_node(comment_list_node),
        None => Ok(vec![]),
    }
}

fn parse_comments_from_comment_list_node(
    comment_list_node: Node,
) -> Result<Vec<ThreadedComment>, XlsxError> {
    let mut threaded: Vec<ThreadedComment> = Vec::new();
    for comment_node in comment_list_node
        .children()
        .filter(|n| n.has_tag_name("threadedComment"))
    {
        let rref = comment_node.attribute("ref");
        let dt = comment_node
            .attribute("dT")
            .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M:%S%.f").ok());
        let person_id = comment_node
            .attribute("personId")
            .ok_or_else(|| XlsxError::Xml("Missing personId attribute".to_string()))
            .and_then(|p| str_to_uuid(p))?;
        let id = comment_node
            .attribute("id")
            .ok_or_else(|| XlsxError::Xml("Missing personId attribute".to_string()))
            .and_then(|p| str_to_uuid(p))?;
        let done = comment_node
            .attribute("id")
            .map(|p| p.to_lowercase() == "true");
        let parent_id = comment_node
            .attribute("parentId")
            .and_then(|p| str_to_uuid(p).ok());
        let mut ext_lst = vec![];
        if let Some(ext_lst_node) = comment_node.children().find(|n| n.has_tag_name("extLst")) {
            for ext in ext_lst_node.children().filter(|n| n.is_element()) {
                if let Some(text) = ext.text() {
                    ext_lst.push(text.to_string());
                }
            }
        }

        let text = {
            if let Some(text_node) = comment_node.children().find(|n| n.has_tag_name("text")) {
                if let Some(txt) = text_node.text() {
                    txt.to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        };
        let mentions = parse_mentions_from_comment_node(comment_node)?;
        threaded.push(ThreadedComment::new(
            text, mentions, ext_lst, rref, dt, person_id, id, parent_id, done,
        ))
    }
    Ok(vec![])
}

fn parse_mentions_from_comment_node(comment_node: Node) -> Result<Vec<Mention>, XlsxError> {
    match comment_node.children().find(|n| n.has_tag_name("mentions")) {
        Some(mentions_node) => {
            let mut mentions: Vec<Mention> = vec![];
            for mention_node in mentions_node.children() {
                let mention_person_id = mention_node
                    .attribute("mentionpersonId")
                    .ok_or_else(|| XlsxError::Xml("Missing mentionpersonId attribute".to_string()))
                    .and_then(|p| str_to_uuid(p))?;
                let mention_id = mention_node
                    .attribute("mentionId")
                    .ok_or_else(|| XlsxError::Xml("Missing mentionId attribute".to_string()))
                    .and_then(|p| str_to_uuid(p))?;
                let start_index = mention_node
                    .attribute("startIndex")
                    .ok_or_else(|| XlsxError::Xml("Missing startIndex attribute".to_string()))
                    .and_then(|p| {
                        p.parse::<u32>()
                            .map_err(|_| XlsxError::Xml("Can't make u32".to_string()))
                    })?;
                let length = mention_node
                    .attribute("length")
                    .ok_or_else(|| XlsxError::Xml("Missing length attribute".to_string()))
                    .and_then(|p| {
                        p.parse::<u32>()
                            .map_err(|_| XlsxError::Xml("Can't make u32".to_string()))
                    })?;
                mentions.push(Mention::new(
                    mention_person_id,
                    mention_id,
                    start_index,
                    length,
                ))
            }
            Ok(mentions)
        }
        None => Ok(vec![]),
    }
}
