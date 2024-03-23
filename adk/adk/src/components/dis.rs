use adk_derive::CDebug;


#[derive(Debug, Default, Clone, CDebug)]
pub struct DisConfig {
    foo: String,
}