
use openai::chat::{ChatCompletionMessage, ChatCompletionMessageRole};

#[derive(Debug, Clone)]
pub struct Message {
  pub role: Role,
  pub content: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Role {
  System,
  User,
}

impl Message {
  pub fn new(role: Role, content: impl Into<String>) -> Self {
    Self { role, content: content.into() }
  }

  pub fn system(content: impl Into<String>) -> Self {
    Self::new(Role::System, content)
  }

  pub fn user(content: impl Into<String>) -> Self {
    Self::new(Role::User, content)
  }
}

impl From<Message> for ChatCompletionMessage {
  fn from(m: Message) -> Self {
    ChatCompletionMessage {
      role: m.role.into(),
      content: Some(m.content),
      name: None,
      function_call: None,
      tool_call_id: None,
      tool_calls: None,
    }
  }
}

impl From<Role> for ChatCompletionMessageRole {
  fn from(r: Role) -> Self {
    match r {
      Role::System => ChatCompletionMessageRole::System,
      Role::User => ChatCompletionMessageRole::User,
    }
  }
}
