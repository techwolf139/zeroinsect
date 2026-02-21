use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchFile {
    pub path: String,
    pub nodes: Vec<LaunchNode>,
    pub parameters: HashMap<String, String>,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchNode {
    pub name: String,
    pub package: String,
    pub executable: String,
    pub namespace: Option<String>,
    pub parameters: HashMap<String, String>,
    pub remappings: HashMap<String, String>,
}

pub fn parse_launch_file(path: &str) -> Result<LaunchFile> {
    let content = std::fs::read_to_string(path)?;
    parse_launch_content(path, &content)
}

pub fn parse_launch_content(path: &str, content: &str) -> Result<LaunchFile> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut nodes = Vec::new();
    let mut parameters = HashMap::new();
    let mut arguments = Vec::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "node" {
                    let mut node = LaunchNode {
                        name: String::new(),
                        package: String::new(),
                        executable: String::new(),
                        namespace: None,
                        parameters: HashMap::new(),
                        remappings: HashMap::new(),
                    };

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => node.name = value,
                            "package" => node.package = value,
                            "exec" => node.executable = value,
                            "ns" => node.namespace = Some(value),
                            _ => {}
                        }
                    }
                    if !node.name.is_empty() {
                        nodes.push(node);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "node" {
                    let mut node = LaunchNode {
                        name: String::new(),
                        package: String::new(),
                        executable: String::new(),
                        namespace: None,
                        parameters: HashMap::new(),
                        remappings: HashMap::new(),
                    };

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => node.name = value,
                            "package" => node.package = value,
                            "exec" => node.executable = value,
                            "ns" => node.namespace = Some(value),
                            _ => {}
                        }
                    }
                    if !node.name.is_empty() {
                        nodes.push(node);
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(LaunchFile {
        path: path.to_string(),
        nodes,
        parameters,
        arguments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_launch() {
        let xml = r#"
        <launch>
            <node name="talker" package="demo_nodes_cpp" exec="talker" />
            <node name="listener" package="demo_nodes_cpp" exec="listener" ns="/test" />
        </launch>
        "#;

        let result = parse_launch_content("test.launch", xml).unwrap();

        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[0].name, "talker");
        assert_eq!(result.nodes[0].package, "demo_nodes_cpp");
        assert_eq!(result.nodes[0].executable, "talker");
        assert_eq!(result.nodes[1].namespace, Some("/test".to_string()));
    }
}
