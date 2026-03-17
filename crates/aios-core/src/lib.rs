pub mod models {
    use serde::{Serialize, Deserialize};
    use std::collections::HashMap;

    /// An intent is the high-level goal an agent or human is trying to achieve.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Intent {
        pub raw_text: String,
        pub target_capability: Option<String>,
        pub parameters: HashMap<String, String>,
    }

    /// System context that is injected into executing commands (MCP analog).
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SystemContext {
        pub active_directory: String,
        pub user_id: String,
        pub permissions: Vec<String>,
    }

    /// The result of an executed capability.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecutionResult {
        pub success: bool,
        pub output: String,
        pub error: Option<String>,
    }
}

pub mod plugin {
    use super::models::{Intent, SystemContext, ExecutionResult};

    /// Represents an AIOS Native Application or Plugin.
    /// Replaces the traditional graphical app with an API/CLI representation.
    pub trait AiosNativeApp {
        /// The unique system identifier for this app (e.g., "core.fs")
        fn id(&self) -> &str;
        
        /// What this app can do. Exposed to the LLM/Agent.
        fn describe_capabilities(&self) -> Vec<String>;

        /// The entrypoint for an agent to execute an intent.
        fn execute(&self, intent: &Intent, context: &SystemContext) -> ExecutionResult;
    }
}

pub fn init_core() {
    println!("AIOS Core initialized. Ready to load plugins.");
}
