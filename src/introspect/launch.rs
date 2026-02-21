use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Parameter value types in ROS2 launch files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ParamValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Yaml(String),
}

impl Default for ParamValue {
    fn default() -> Self {
        ParamValue::String(String::new())
    }
}

impl std::fmt::Display for ParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamValue::String(s) => write!(f, "{}", s),
            ParamValue::Int(i) => write!(f, "{}", i),
            ParamValue::Double(d) => write!(f, "{}", d),
            ParamValue::Bool(b) => write!(f, "{}", b),
            ParamValue::Yaml(y) => write!(f, "{}", y),
        }
    }
}

/// Topic/parameter remapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remap {
    pub from: String,
    pub to: String,
}

/// Global argument in launch file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArgument {
    pub name: String,
    pub default: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchFile {
    pub path: String,
    pub nodes: Vec<LaunchNode>,
    pub parameters: HashMap<String, ParamValue>,
    pub remaps: Vec<Remap>,
    pub arguments: Vec<LaunchArgument>,
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
    let mut remaps = Vec::new();
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
                } else if tag == "param" {
                    let mut name = String::new();
                    let mut value = String::new();
                    let mut param_type = String::from("str");

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => name = val,
                            "value" => value = val,
                            "type" => param_type = val,
                            _ => {}
                        }
                    }

                    if !name.is_empty() {
                        let param_value = parse_param_value(&value, &param_type);
                        parameters.insert(name, param_value);
                    }
                } else if tag == "remap" {
                    let mut from = String::new();
                    let mut to = String::new();

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "from" => from = val,
                            "to" => to = val,
                            _ => {}
                        }
                    }

                    if !from.is_empty() && !to.is_empty() {
                        remaps.push(Remap { from, to });
                    }
                } else if tag == "arg" {
                    let mut name = String::new();
                    let mut default = None;
                    let mut description = None;

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => name = val,
                            "default" => default = Some(val),
                            "description" => description = Some(val),
                            _ => {}
                        }
                    }

                    if !name.is_empty() {
                        arguments.push(LaunchArgument {
                            name,
                            default,
                            description,
                        });
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
                } else if tag == "param" {
                    let mut name = String::new();
                    let mut value = String::new();
                    let mut param_type = String::from("str");

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => name = val,
                            "value" => value = val,
                            "type" => param_type = val,
                            _ => {}
                        }
                    }

                    if !name.is_empty() {
                        let param_value = parse_param_value(&value, &param_type);
                        parameters.insert(name, param_value);
                    }
                } else if tag == "remap" {
                    let mut from = String::new();
                    let mut to = String::new();

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "from" => from = val,
                            "to" => to = val,
                            _ => {}
                        }
                    }

                    if !from.is_empty() && !to.is_empty() {
                        remaps.push(Remap { from, to });
                    }
                } else if tag == "arg" {
                    let mut name = String::new();
                    let mut default = None;
                    let mut description = None;

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => name = val,
                            "default" => default = Some(val),
                            "description" => description = Some(val),
                            _ => {}
                        }
                    }

                    if !name.is_empty() {
                        arguments.push(LaunchArgument {
                            name,
                            default,
                            description,
                        });
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
        remaps,
        arguments,
    })
}

fn parse_param_value(value: &str, param_type: &str) -> ParamValue {
    match param_type {
        "int" => value
            .parse()
            .map(ParamValue::Int)
            .unwrap_or_else(|_| ParamValue::String(value.to_string())),
        "double" | "float" => value
            .parse()
            .map(ParamValue::Double)
            .unwrap_or_else(|_| ParamValue::String(value.to_string())),
        "bool" => {
            let lower = value.to_lowercase();
            if lower == "true" || lower == "1" {
                ParamValue::Bool(true)
            } else if lower == "false" || lower == "0" {
                ParamValue::Bool(false)
            } else {
                ParamValue::String(value.to_string())
            }
        }
        "yaml" => ParamValue::Yaml(value.to_string()),
        _ => ParamValue::String(value.to_string()),
    }
}

pub fn scan_launch_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();

    if !dir.is_dir() {
        return results;
    }

    let entries = std::fs::read_dir(dir).ok();
    if let Some(entries) = entries {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(scan_launch_files(&path));
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "xml" || ext_str == "py" {
                    if let Some(filename) = path.file_name() {
                        let name = filename.to_string_lossy().to_lowercase();
                        if name.contains("launch") {
                            results.push(path);
                        }
                    }
                }
            }
        }
    }

    results
}

pub fn discover_workspace() -> Option<PathBuf> {
    let candidates = vec![
        PathBuf::from("."),
        PathBuf::from(".."),
        PathBuf::from("/opt/ros"),
    ];

    for base in candidates {
        if base == PathBuf::from(".") || base == PathBuf::from("..") {
            if let Ok(cwd) = std::env::current_dir() {
                for ancestor in cwd.ancestors() {
                    if let Some(ws) = check_workspace(ancestor) {
                        return Some(ws);
                    }
                }
            }
        } else if base.exists() {
            if let Some(ws) = check_workspace(&base) {
                return Some(ws);
            }
        }
    }

    None
}

fn check_workspace(path: &Path) -> Option<PathBuf> {
    let workspace_markers = [
        "colcon_workspace",
        "ament_workspace",
        "install",
        "build",
        "src",
    ];

    for marker in workspace_markers {
        let marker_path = path.join(marker);
        if marker_path.exists() && marker_path.is_dir() {
            return Some(path.to_path_buf());
        }
    }

    None
}

pub fn find_packages(workspace: &Path) -> Vec<PathBuf> {
    let mut packages = Vec::new();
    let src_dir = workspace.join("src");

    if !src_dir.exists() {
        return packages;
    }

    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let package_xml = path.join("package.xml");
                let cmake_lists = path.join("CMakeLists.txt");

                if package_xml.exists() || cmake_lists.exists() {
                    packages.push(path);
                }
            }
        }
    }

    packages
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

    #[test]
    fn test_parse_param_tags() {
        let xml = r#"
        <launch>
            <param name="robot_description" value="$(find robot_description)/robot.urdf" type="str"/>
            <param name="max_velocity" value="1.5" type="double"/>
            <param name="use_sim_time" value="true" type="bool"/>
            <param name="num_threads" value="4" type="int"/>
        </launch>
        "#;

        let result = parse_launch_content("test.launch", xml).unwrap();

        assert_eq!(result.parameters.len(), 4);
        assert!(matches!(
            result.parameters.get("robot_description"),
            Some(ParamValue::String(_))
        ));
        assert!(matches!(
            result.parameters.get("max_velocity"),
            Some(ParamValue::Double(_))
        ));
        assert!(matches!(
            result.parameters.get("use_sim_time"),
            Some(ParamValue::Bool(true))
        ));
        assert!(matches!(
            result.parameters.get("num_threads"),
            Some(ParamValue::Int(4))
        ));
    }

    #[test]
    fn test_parse_remap_tags() {
        let xml = r#"
        <launch>
            <remap from="/scan" to="/front/scan"/>
            <remap from="/cmd_vel" to="/diff_drive/cmd_vel"/>
        </launch>
        "#;

        let result = parse_launch_content("test.launch", xml).unwrap();

        assert_eq!(result.remaps.len(), 2);
        assert_eq!(result.remaps[0].from, "/scan");
        assert_eq!(result.remaps[0].to, "/front/scan");
        assert_eq!(result.remaps[1].from, "/cmd_vel");
    }

    #[test]
    fn test_parse_arguments() {
        let xml = r#"
        <launch>
            <arg name="use_sim_time" default="true" description="Use simulation time"/>
            <arg name="robot_name" default="robot" description="Name of the robot"/>
        </launch>
        "#;

        let result = parse_launch_content("test.launch", xml).unwrap();

        assert_eq!(result.arguments.len(), 2);
        assert_eq!(result.arguments[0].name, "use_sim_time");
        assert_eq!(result.arguments[0].default, Some("true".to_string()));
        assert_eq!(
            result.arguments[0].description,
            Some("Use simulation time".to_string())
        );
    }
}
