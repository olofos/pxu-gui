#[derive(serde::Deserialize, serde::Serialize)]
pub struct Figure {
    pub paths: Vec<pxu::Path>,
    pub state: pxu::State,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FigureDescription {
    pub name: String,
    pub description: String,
    pub filename: String,
}
