pub mod analysis;
pub mod device;
pub mod llm;
pub mod workflow;

// Re-export commonly used types
pub use analysis::{BoundingBox, QuestionContext};
pub use device::{
    keyboard::Keyboard,
    pen::Pen,
    screenshot::Screenshot,
    touch::{Touch, TriggerCorner},
    DeviceModel,
};
pub use llm::{openai::OpenAI, LLMEngine};
pub use workflow::{orchestrator::Orchestrator, Workflow};

