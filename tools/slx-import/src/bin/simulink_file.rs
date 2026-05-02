use serde;
use serde::Deserialize;
use serde_with::rust::deserialize_ignore_any;

#[derive(Debug, Deserialize)]
pub struct Param {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "$value", default)]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct InstanceData {}

#[derive(Debug, Deserialize)]
pub struct PortProperties {}

#[derive(Debug, Deserialize)]
pub struct Mask {}
#[derive(Debug, Deserialize)]
pub struct List {}

#[derive(Debug, Deserialize)]
pub struct SysRef {
    #[serde(rename = "Ref")]
    pub sys_ref: String,
}

#[derive(Debug, Deserialize)]
pub enum BlockItem {
    P(Param),
    PortCounts(String),
    InstanceData(InstanceData),
    Mask(Mask),
    PortProperties(PortProperties),
    System(SysRef),
    List(List),
    #[serde(other, deserialize_with = "deserialize_ignore_any")]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Block {
    #[serde(rename = "BlockType")]
    pub block_type: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(default)]
    #[serde(rename = "$value")]
    pub items: Vec<BlockItem>,
}

#[derive(Debug, Deserialize)]
pub enum LineItem {
    P(Param),
    #[serde(other, deserialize_with = "deserialize_ignore_any")]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Line {
    #[serde(rename = "$value")]
    pub items: Vec<LineItem>,
}

#[derive(Debug, Deserialize)]
pub struct Annotation {}

#[derive(Debug, Deserialize)]
pub enum SysItem {
    P(Param),
    Block(Block),
    Line(Line),
    Annotation(Annotation),
    #[serde(other, deserialize_with = "deserialize_ignore_any")]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct System {
    #[serde(rename = "$value")]
    pub items: Vec<SysItem>,
}

#[derive(Debug, Deserialize)]
pub struct SystemRef {
    #[serde(rename = "Ref")]
    pub sysref: String,
}

#[derive(Debug, Deserialize)]
pub enum LibraryItem {
    P(Param),
    System(SystemRef),
    #[serde(other, deserialize_with = "deserialize_ignore_any")]
    Other,
}

#[derive(Debug, Deserialize, Default)]
pub struct Library {
    #[serde(default)]
    #[serde(rename = "$value")]
    pub items: Vec<LibraryItem>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Model {
    #[serde(default)]
    #[serde(rename = "$value")]
    pub items: Vec<LibraryItem>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SubSystem {
    #[serde(default)]
    #[serde(rename = "$value")]
    pub items: Vec<LibraryItem>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInformation {
    // this may be empty, add a default
    #[serde(default)]
    pub Library: Library,
    #[serde(default)]
    pub Model: Model,
    #[serde(default)]
    pub SubSystem: SubSystem,
}
