pub const PROMPT_INTENT_PARSING: &str = r#"You are a robot command parser. 
Given a natural language command, extract the intent and parameters.

Output JSON:
{
  "intent": "MoveArm | GetStatus | Navigate | Stop | Custom",
  "parameters": {},
  "confidence": 0.0-1.0,
  "needs_confirmation": true/false
}"#;

pub const PROMPT_SCHEMA_MATCHING: &str = r#"You are a ROS message schema expert. 
Given a ROS1 message definition and ROS2 message definition, 
determine if they are semantically equivalent and can be auto-converted.

Output JSON:
{
  "compatible": true/false,
  "confidence": 0.0-1.0,
  "field_mappings": [{"ros1": "field_a", "ros2": "field_b", "type_convert": "..."}],
  "issues": ["any conversion problems"]
}"#;
